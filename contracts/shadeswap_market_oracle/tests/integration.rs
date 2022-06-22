use std::collections::HashMap;
use cosmwasm_std::{ HumanAddr, Uint128, to_binary };

use fadroma::{
    scrt::ContractLink, 
    ensemble::{ MockEnv, ContractEnsemble },
};

use shade_oracles_ensemble::harness::{
    MockBand,
    OracleRouter,
    ProxyBandOracle,
    ShadeMarketOracle,
    MockShadePair,
    Snip20,
};

use shade_oracles::{
    common::{Contract, OraclePrice},
    band::{self, proxy},
    router,
    shadeswap_market_oracle,
};

use mock_shade_pair::contract as mock_shade_pair;

fn basic_market_test(
    symbol: String, 
    base_peg: Option<String>,
    prices: HashMap<String, Uint128>,
    primary_symbol: String,
    primary_pool: Uint128,
    primary_decimals: u32,
    base_symbol: String,
    base_pool: Uint128,
    base_decimals: u32,
    expected: Uint128,
) {
    let mut ensemble = ContractEnsemble::new(50);

    let reg_router = ensemble.register(Box::new(OracleRouter));
    let reg_market_oracle = ensemble.register(Box::new(ShadeMarketOracle));
    let reg_mock_band = ensemble.register(Box::new(MockBand));
    let reg_mock_band_proxy = ensemble.register(Box::new(ProxyBandOracle));
    let reg_shade_pair = ensemble.register(Box::new(MockShadePair));
    let reg_snip20 = ensemble.register(Box::new(Snip20));

    let band = ensemble.instantiate(
        reg_mock_band.id,
        &band::InitMsg { },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("band".into()),
                code_hash: reg_mock_band.code_hash.clone(),
            }
        )
    ).unwrap().instance;

    let band_proxy = ensemble.instantiate(
        reg_mock_band_proxy.id,
        &proxy::InitMsg {
            owner: HumanAddr("admin".into()),
            band: Contract {
                address: band.address.clone(),
                code_hash: band.code_hash.clone(),
            },
            quote_symbol: "USD".to_string(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("band_proxy".into()),
                code_hash: reg_mock_band.code_hash.clone(),
            }
        )
    ).unwrap().instance;

    let router = ensemble.instantiate(
        reg_router.id,
        &router::InitMsg {
            owner: HumanAddr("admin".into()),
            default_oracle: Contract {
                address: band_proxy.address.clone(),
                code_hash: band_proxy.code_hash.clone(),
            },
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("router".into()),
                code_hash: reg_router.code_hash.clone(),
            }
        )
    ).unwrap().instance;

    // Configure mock band prices
    for (sym, price) in prices.clone() {
        ensemble.execute(
            &band::HandleMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: "USD".to_string(),
                rate: price,
                last_updated: None,
            },
            MockEnv::new(
                "admin", 
                band.clone(),
            ),
        ).unwrap();
    }

    // Setup tokens
    let primary_token = ensemble
        .instantiate(
            reg_snip20.id,
            &snip20_reference_impl::msg::InitMsg {
                name: "Primary".into(),
                admin: Some("admin".into()),
                symbol: primary_symbol,
                decimals: primary_decimals as u8,
                initial_balances: None,
                prng_seed: to_binary("").ok().unwrap(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("primary_token".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }),
        )
        .unwrap().instance;

    let base_token = ensemble
        .instantiate(
            reg_snip20.id,
            &snip20_reference_impl::msg::InitMsg {
                name: "Base".into(),
                admin: Some("admin".into()),
                symbol: base_symbol,
                decimals: base_decimals as u8,
                initial_balances: None,
                prng_seed: to_binary("").ok().unwrap(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("base_token".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }),
        )
        .unwrap().instance;

    let shade_pair = ensemble.instantiate(
        reg_shade_pair.id,
        &mock_shade_pair::InitMsg { },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("shade_pair".into()),
                code_hash: reg_shade_pair.code_hash.clone(),
            }
        )
    ).unwrap().instance;

    //assert!(false, "HERE");

    ensemble.execute(
        &mock_shade_pair::HandleMsg::MockPool {
            token_a: Contract {
                address: primary_token.address,
                code_hash: primary_token.code_hash,
            },
            amount_a: primary_pool,
            token_b: Contract {
                address: base_token.address,
                code_hash: base_token.code_hash,
            },
            amount_b: base_pool,
        },
        MockEnv::new(
            "admin",
            shade_pair.clone(),
        ),
    ).unwrap();
    assert!(false, "HERE2");

    let market_oracle = ensemble.instantiate(
        reg_market_oracle.id,
        &shadeswap_market_oracle::InitMsg {
            admins: None,
            router: Contract {
                address: router.address.clone(),
                code_hash: router.code_hash.clone(),
            },
            pair: Contract {
                address: shade_pair.address.clone(),
                code_hash: shade_pair.code_hash.clone(),
            },
            symbol: symbol.clone(),
            base_peg: base_peg.clone(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("market".into()),
                code_hash: reg_market_oracle.code_hash.clone(),
            }
        )
    ).unwrap().instance;

    // Configure router w/ market oracle
    ensemble.execute(
        &router::HandleMsg::UpdateRegistry {
            operation: router::RegistryOperation::Add {
                oracle: Contract {
                    address: market_oracle.address.clone(),
                    code_hash: market_oracle.code_hash.clone(),
                },
                key: symbol.clone(),
            }
        },
        MockEnv::new(
            "admin",
            router.clone(),
        ),
    ).unwrap();

    match ensemble.query(
        market_oracle.address.clone(),
        &shadeswap_market_oracle::QueryMsg::GetPrice {
            key: symbol.clone()
        },
    ).unwrap() {
        OraclePrice { key: _, price } => assert_eq!(expected, price.rate, "Expected: {} Got: {}", expected, price.rate),
    };
}

macro_rules! basic_market_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (symbol, base_peg, prices,
                     primary_symbol, primary_pool, primary_decimals,
                     base_symbol, base_pool, base_decimals, expected) = $value;

                basic_market_test(
                    symbol.to_string(), 
                    base_peg,
                    prices.into_iter().map(|(sym, p)| (sym.to_string(), Uint128(p))).collect(),
                    primary_symbol.to_string(),
                    Uint128(primary_pool),
                    primary_decimals,
                    base_symbol.to_string(),
                    Uint128(base_pool),
                    base_decimals,
                    Uint128(expected),
                );
            }
        )*
    }
}

basic_market_tests! {
    usd_stables_same_decimals: (
        "USDT",
        None,
        vec![
            ("USDT", 10u128.pow(18)), // $1
            ("USDC", 10u128.pow(18)), // $1
        ],
        "USDT",
        10u128.pow(10),
        6,
        "USDC",
        10u128.pow(10),
        6,
        0_999901 * 10u128.pow(12), // ~$.99 (slippage)
    ),
    usd_stables_diff_decimals: (
        "USDT",
        None,
        vec![
            ("USDT", 10u128.pow(18)), // $1
            ("USDC", 10u128.pow(18)), // $1
        ],
        "USDT",
        10u128.pow(16),
        13,
        "USDC",
        10u128.pow(21),
        18,
        999000999000999001, // ~$.99 (slippage)
    ),
    shd_sscrt_mainnet_test: (
        "SHD",
        None,
        vec![
            ("SSCRT", 979575 * 10u128.pow(12)), // $0.979575
        ],

        "SHD",
        10253_22 * 10u128.pow(6),
        8,

        "SSCRT",
        86770_13 * 10u128.pow(4),
        6,

        8_289_060_794_625 * 10u128.pow(6), // ~$8.28
    ),
}
