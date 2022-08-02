use cosmwasm_std::Uint128;
use cosmwasm_std::{to_binary, Addr};
use mock_shade_pair::contract as mock_shade_pair;
use shade_oracles::common::PriceResponse;
use shade_oracles::core::{snip20, Contract};
use shade_oracles::{
    common::{self, InstantiateCommonConfig, OraclePrice},
    core::{ExecuteCallback, InstantiateCallback, Query},
    interfaces::{lp::market as shadeswap_market_oracle, router},
};
use shade_oracles_multi_test::multi::mocks::Snip20;
use shade_oracles_multi_test::{
    helpers::OracleCore,
    multi::{market::shadeswap::ShadeSwapMarketOracle, MockShadePair},
    App, MultiTestable,
};
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
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
    let user = Addr::unchecked("superadmin");
    let mut app = App::default();

    let oracle_core = OracleCore::setup(&mut app, prices).unwrap();
    let router = oracle_core.router;

    // Setup tokens
    let primary_token = snip20::InstantiateMsg {
        name: "Primary".into(),
        admin: Some("superadmin".into()),
        symbol: primary_symbol,
        decimals: primary_decimals as u8,
        initial_balances: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: None,
    }
    .test_init(
        Snip20::default(),
        &mut app,
        user.clone(),
        "primary_token",
        &[],
    )
    .unwrap();

    let base_token = snip20::InstantiateMsg {
        name: "Base".into(),
        admin: Some("superadmin".into()),
        symbol: base_symbol,
        decimals: base_decimals as u8,
        initial_balances: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: None,
    }
    .test_init(Snip20::default(), &mut app, user.clone(), "base_token", &[])
    .unwrap();

    let shade_pair = mock_shade_pair::InstantiateMsg {}
        .test_init(
            MockShadePair::default(),
            &mut app,
            user.clone(),
            "shade_pair",
            &[],
        )
        .unwrap();

    mock_shade_pair::ExecuteMsg::MockPool {
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
    }
    .test_exec(&shade_pair, &mut app, user.clone(), &[])
    .unwrap();

    let market_oracle = shadeswap_market_oracle::InstantiateMsg {
        config: InstantiateCommonConfig {
            supported_keys: None,
            router: router.clone().into(),
            enabled: true,
            only_band: true,
        },
        base_peg,
        symbol: symbol.clone(),
        pair: shade_pair.clone().into(),
    }
    .test_init(
        ShadeSwapMarketOracle::default(),
        &mut app,
        user.clone(),
        "shade-swap-market-oracle",
        &[],
    )
    .unwrap();

    // Configure router w/ market oracle

    router::ExecuteMsg::UpdateRegistry {
        operation: router::RegistryOperation::Replace {
            oracle: Contract {
                address: market_oracle.address.clone(),
                code_hash: market_oracle.code_hash.clone(),
            },
            key: symbol.clone(),
        },
    }
    .test_exec(&router, &mut app, user.clone(), &[])
    .unwrap();

    let price: PriceResponse = common::OracleQuery::GetPrice { key: symbol.clone() }
        .test_query(&market_oracle, &app)
        .unwrap();
    let data = price.price.data();
    assert_eq!(
        expected, data.rate,
        "Expected: {} Got: {}",
        expected, data.rate
    );
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
                    prices.into_iter().map(|(sym, p)| (sym.to_string(), Uint128::from(p))).collect(),
                    primary_symbol.to_string(),
                    Uint128::from(primary_pool),
                    primary_decimals,
                    base_symbol.to_string(),
                    Uint128::from(base_pool),
                    base_decimals,
                    Uint128::from(expected)
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
        999901 * 10u128.pow(12), // ~$.99 (slippage)
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
        999000999000999001u128, // ~$.99 (slippage)
    ),
    shd_sscrt_mainnet_test: (
        "SHD",
        None,
        vec![
            ("SSCRT", 979575 * 10u128.pow(12)), // $0.979575
        ],

        "SHD",
        1_025_322 * 10u128.pow(6),
        8,

        "SSCRT",
        8_677_013 * 10u128.pow(4),
        6,

        8_289_060_794_625 * 10u128.pow(6), // ~$8.28
    ),
}
