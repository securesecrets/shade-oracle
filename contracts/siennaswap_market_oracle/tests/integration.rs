use std::collections::HashMap;
use cosmwasm_std::{ HumanAddr, Uint128 };

use fadroma::{
    ContractLink, 
    ensemble::{ MockEnv, ContractEnsemble },
};

use shade_oracles_ensemble::harness::{
    MockBand,
    OracleRouter,
    ProxyBandOracle,
    SiennaMarketOracle,
};

use contract_harness::{
    MockSiennaPair,
};

use shade_oracles::{
    common::{Contract, OraclePrice},
    band::{self, proxy},
    router,
    sienna_market_oracle,
};

fn basic_market_test(
    symbol: String, 
    base_peg: Option<String>,
    prices: HashMap<String, Uint128>,
    primary_symbol: String,
    primary_pool: Uint128,
    primary_decimals: Uint128,
    base_symbol: String,
    base_pool: Uint128,
    base_decimals: Uint128,
    expected: Uint128,
) {
    let mut ensemble = ContractEnsemble::new(50);

    let reg_router = ensemble.register(Box::new(OracleRouter));
    let reg_market_oracle = ensemble.register(Box::new(SiennaMarketOracle));
    let reg_mock_band = ensemble.register(Box::new(MockBand));
    let reg_mock_band_proxy = ensemble.register(Box::new(ProxyBandOracle));
    let reg_sienna_pair = ensemble.register(Box::new(MockSiennaPair));

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
    ).unwrap();

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
    ).unwrap();

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
    ).unwrap();


    let mut operations = vec![];

    for (sym, _) in prices.clone() {
        operations.push(
            router::RegistryOperation::Add {
                oracle: Contract {
                    address: band_proxy.address.clone(),
                    code_hash: band_proxy.code_hash.clone(),
                },
                key: sym,
            }
        );
    }

    // Configure BAND symbols on router
    ensemble.execute(
        &router::HandleMsg::BatchUpdateRegistry {
            operations,
        },
        MockEnv::new(
            "admin",
            router.clone(),
        ),
    ).unwrap();

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

    let market_oracle = ensemble.instantiate(
        reg_market_oracle.id,
        &sienna_market_oracle::InitMsg {
            admins: None,
            router: Contract {
                address: router.address.clone(),
                code_hash: router.code_hash.clone(),
            },
            pair: Contract {
                address: pair.address.clone(),
                code_hash: pair.code_hash.clone(),
            },
            symbol: symbol.clone(),
            base_peg: base_peg.clone(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("market".into()),
                code_hash: market_oracle.code_hash.clone(),
            }
        )
    ).unwrap();

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
        &market_oracle::QueryMsg::GetPrice {
            key: symbol.clone()
        }
    ).unwrap() {
        OraclePrice { key: _, price } => assert_eq!(expected, price, "Expected: {} Got: {}", expected, price),
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
                    Uint128(primary_decimals),
                    base_symbol.to_string(),
                    Uint128(base_pool),
                    Uint128(base_decimals),
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
            ("USDT", 10u128.pow(18)), // 1$
            ("USDC", 10u128.pow(18)), // 1$
        ],
        "USDT",
        10u128.pow(100),
        6,
        "USDC",
        10u128.pow(100), // $1
        6,
        10u128.pow(18),
    ),
    usd_stables_diff_decimals: (
        "USDT",
        None,
        vec![
            ("USDT", 10u128.pow(18)), // 1$
            ("USDC", 10u128.pow(18)), // 1$
        ],
        "USDT",
        10u128.pow(100),
        13,
        "USDC",
        10u128.pow(100), // $1
        18,
        10u128.pow(18),
    ),
}
