use shade_oracles::{
    common::{InstantiateCommonConfig, OraclePrice, PriceResponse},
    core::{ExecuteCallback, InstantiateCallback, Query},
    interfaces::band::{self},
    interfaces::index_oracle,
    interfaces::router,
};
use shade_oracles_multi_test::{
    helpers::OracleCore, multi::index::IndexOracle, App, MultiTestable,
};
use std::collections::HashMap;

use cosmwasm_std::{Addr, Uint128};

fn basic_index_test(
    symbol: String,
    basket: Vec<(String, Uint128)>,
    prices: HashMap<String, Uint128>,
    target: Uint128,
    expected: Uint128,
    error: Uint128,
) {
    let user = Addr::unchecked("superadmin");
    let mut app = App::default();

    let oracle_core = OracleCore::setup(&mut app, prices).unwrap();
    let router = oracle_core.router;

    let index_oracle = index_oracle::InstantiateMsg {
        config: InstantiateCommonConfig::new(None, router.clone().into(), true, false),
        symbol: symbol.clone(),
        target,
        basket,
    }
    .test_init(
        IndexOracle::default(),
        &mut app,
        user.clone(),
        "index-oracle",
        &[],
    )
    .unwrap();

    // Configure router w/ index oracle
    router::ExecuteMsg::UpdateRegistry {
        operation: router::RegistryOperation::Add {
            oracle: index_oracle.into(),
            key: symbol.clone(),
        },
    }
    .test_exec(&router, &mut app, user, &[])
    .unwrap();

    let price: PriceResponse = index_oracle::QueryMsg::GetPrice { key: symbol }
        .test_query(&router, &app)
        .unwrap();
    let data = price.price.data();

    {
        let err = if data.rate > expected {
            data.rate - expected
        } else {
            expected - data.rate
        };
        let acceptable = expected.multiply_ratio(error, 10u128.pow(18));

        assert!(
            err <= acceptable,
            "price: {}, expected: {}, exceeds acceptable error",
            data.rate,
            expected
        );
    };
}

macro_rules! basic_index_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (symbol, basket, prices, target, expected, error) = $value;
                basic_index_test(
                    symbol.to_string(),
                    basket.into_iter().map(|(sym, w)| (sym.to_string(), Uint128::from(w))).collect(),
                    prices.into_iter().map(|(sym, p)| (sym.to_string(), Uint128::from(p))).collect(),
                    Uint128::from(target),
                    Uint128::from(expected),
                    Uint128::from(error),
                );
            }
        )*
    }
}

basic_index_tests! {
    basic_index_test_0: (
        "sUSD",
        vec![
            ("USD", 10u128.pow(18)), // 100%
        ],
        vec![
            ("USD", 10u128.pow(18)), // $1
        ],
        10u128.pow(18), // $1
        10u128.pow(18), // $1
        10u128.pow(10), // .000001% error
    ),
    basic_index_test_1: (
        "INDEX",
        vec![
            ("USD", 25 * 10u128.pow(16)), // 25%
            ("BTC", 25 * 10u128.pow(16)), // 25%
            ("ETH", 25 * 10u128.pow(16)), // 25%
            ("XAU", 25 * 10u128.pow(16)), // 25%
        ],
        vec![
            ("USD", 1_00 * 10u128.pow(14)), // $1
            ("BTC", 29_398_20 * 10u128.pow(14)), // $29398.2
            ("ETH", 1_831_26 * 10u128.pow(14)), // $1831.26
            ("XAU", 1_852_65 * 10u128.pow(14)), // $1852.65
        ],
        1 * 10u128.pow(18),
        1 * 10u128.pow(18),
        10u128.pow(10), // .000001% error
    ),
    basic_index_test_2: (
        "SILK",
        vec![
            ("USD", 39_33 * 10u128.pow(14)), //  39.32%
            ("CNY", 7_13 * 10u128.pow(14)), //  7.13%
            ("EUR", 15_97 * 10u128.pow(14)), // 15.97%
            ("JPY", 764 * 10u128.pow(14)), //  7.64%
            ("GBP", 3_40 * 10u128.pow(14)), //  3.4%
            ("CAD", 4_58 * 10u128.pow(14)), //  4.58%
            ("KRW", 1_53 * 10u128.pow(14)), //  1.53%
            ("AUD", 232 * 10u128.pow(14)), //  2.32%
            ("IDR", 2_50 * 10u128.pow(14)), //  2.5%
            ("CHF", 4_44 * 10u128.pow(14)), //  4.44%
            ("SEK", 0_84 * 10u128.pow(14)), //  0.84%
            ("NOK", 0_82 * 10u128.pow(14)), //  0.82%
            ("SGD", 2_50 * 10u128.pow(14)), //  2.5%
            ("XAU", 5_00 * 10u128.pow(14)), //  5.0%
            ("BTC", 2_00 * 10u128.pow(14)), //  2.0%
        ],
        vec![
            ("USD", 1_00 * 10u128.pow(14)), // $1
            ("CNY", 0_15 * 10u128.pow(14)), // $0.15
            ("EUR", 1_07 * 10u128.pow(14)), // $1.07
            ("JPY", 0_0079 * 10u128.pow(12)), // $0.0079
            ("GBP", 1_26 * 10u128.pow(14)), // $1.26
            ("CAD", 0_78 * 10u128.pow(14)), // $0.78
            ("KRW", 0_00079 * 10u128.pow(11)), // $0.00079
            ("AUD", 0_71 * 10u128.pow(14)), // $0.71
            ("IDR", 0_000068 * 10u128.pow(10)), // $0.000068
            ("CHF", 1_04 * 10u128.pow(14)), // $1.04
            ("SEK", 0_10 * 10u128.pow(14)), // $0.10
            ("NOK", 0_10 * 10u128.pow(14)), // $0.10
            ("SGD", 0_73 * 10u128.pow(14)), // $0.73
            ("XAU", 1_852_65 * 10u128.pow(14)), // $1852.65
            ("BTC", 29_398_20 * 10u128.pow(14)), // $29398.2
        ],
        1_05 * 10u128.pow(16),
        1_05 * 10u128.pow(16),
        10u128.pow(10), // .000001% error
    ),
}

/* - Setup oracle with symbol, basket, prices, & target -- check against expected_initial
 * - Change to new_prices & apply mod_basket changes -- check against expected_final
 */
#[allow(clippy::too_many_arguments)]
fn mod_index_test(
    symbol: String,
    basket: Vec<(String, Uint128)>,
    prices: HashMap<String, Uint128>,
    new_prices: HashMap<String, Uint128>,
    mod_basket: Vec<(String, Uint128)>,
    expected_weights: Vec<(String, Uint128)>,
    target: Uint128,
    expected_initial: Uint128,
    expected_final: Uint128,
    error: Uint128,
) {
    let user = Addr::unchecked("superadmin");
    let mut app = App::default();

    let oracle_core = OracleCore::setup(&mut app, prices).unwrap();
    let band = oracle_core.band;
    let band_proxy = oracle_core.band_proxy;
    let router = oracle_core.router;

    let index_oracle = index_oracle::InstantiateMsg {
        config: InstantiateCommonConfig::new(None, router.clone().into(), true, false),
        symbol: symbol.clone(),
        target,
        basket,
    }
    .test_init(
        IndexOracle::default(),
        &mut app,
        user.clone(),
        "index-oracle",
        &[],
    )
    .unwrap();

    // Configure router w/ index oracle
    router::ExecuteMsg::UpdateRegistry {
        operation: router::RegistryOperation::Add {
            oracle: index_oracle.clone().into(),
            key: symbol.clone(),
        },
    }
    .test_exec(&router, &mut app, user.clone(), &[])
    .unwrap();

    let price: PriceResponse = index_oracle::QueryMsg::GetPrice { key: symbol.clone() }
        .test_query(&router, &app)
        .unwrap();
    let data = price.price.data();
    {
        let mut err = Uint128::zero();
        if data.rate > expected_initial {
            err = (data.rate - expected_initial);
        } else {
            err = (expected_initial - data.rate);
        }
        let acceptable = expected_initial.multiply_ratio(error, 10u128.pow(18));

        assert!(
            err <= acceptable,
            "price: {}, expected: {}, exceeds acceptable error",
            data.rate,
            expected_initial
        );
    };

    /* TODO
     * - Update new prices
     * - mod basket
     * - check final price
     */

    // Update mock band prices
    for (sym, price) in new_prices.clone() {
        band::ExecuteMsg::UpdateSymbolPrice {
            base_symbol: sym,
            quote_symbol: "USD".to_string(),
            rate: price,
            last_updated: None,
        }
        .test_exec(&band, &mut app, user.clone(), &[])
        .unwrap();
    }

    let price: PriceResponse = index_oracle::QueryMsg::GetPrice { key: symbol.clone() }
        .test_query(&router, &app)
        .unwrap();
    let data = price.price.data();
    {
        let mut err = Uint128::zero();
        if data.rate > expected_final {
            err = (data.rate - expected_final);
        } else {
            err = (expected_final - data.rate);
        }
        let acceptable = expected_final.multiply_ratio(error, 10u128.pow(18));

        assert!(
            err <= acceptable,
            "Price change check failed price: {}, expected: {}, exceeds acceptable error",
            data.rate,
            expected_final
        );
    };

    // Update basket
    index_oracle::ExecuteMsg::ModBasket { basket: mod_basket }
        .test_exec(&index_oracle, &mut app, user, &[])
        .unwrap();

    // check basket changed
    match (index_oracle::QueryMsg::Basket {}
        .test_query(&index_oracle, &app)
        .unwrap())
    {
        index_oracle::BasketResponse { mut basket } => {
            basket.sort();
            for (sym, w, _) in basket {
                assert!(
                    expected_weights.contains(&(sym.clone(), w.clone())),
                    "Bad symbol found {}, {}",
                    sym,
                    w
                );
            }
        }
    };

    // check price doesn't change on mod_price
    let price: PriceResponse = index_oracle::QueryMsg::GetPrice { key: symbol.clone() }
        .test_query(&router, &app)
        .unwrap();
    let data = price.price.data();
    {
        let mut err = Uint128::zero();
        if data.rate > expected_final {
            err = (data.rate - expected_final);
        } else {
            err = (expected_final - data.rate);
        }
        let acceptable = expected_final.multiply_ratio(error, 10u128.pow(18));

        assert!(
            err <= acceptable,
            "Post-Mod price: {}, expected: {}, exceeds acceptable error",
            data.rate,
            expected_final
        );
    };
}

macro_rules! mod_index_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (symbol, basket, prices, new_prices, mod_basket, expected_weights, target, expected_initial, expected_final, error) = $value;

                mod_index_test(
                    symbol.to_string(),
                    basket.into_iter().map(|(sym, w)| (sym.to_string(), Uint128::from(w))).collect(),
                    prices.into_iter().map(|(sym, p)| (sym.to_string(), Uint128::from(p))).collect(),
                    new_prices.into_iter().map(|(sym, p)| (sym.to_string(), Uint128::from(p))).collect(),
                    mod_basket.into_iter().map(|(sym, p)| (sym.to_string(), Uint128::from(p))).collect(),
                    expected_weights.into_iter().map(|(sym, p)| (sym.to_string(), Uint128::from(p))).collect(),
                    Uint128::from(target),
                    Uint128::from(expected_initial),
                    Uint128::from(expected_final),
                    Uint128::from(error),
                );
            }
        )*
    }
}

mod_index_tests! {
    mod_index_test_0: (
        "sUSD",
        // basket
        vec![
            ("USD", 10u128.pow(18)), // 100%
        ],
        // prices
        vec![
            ("USD", 10u128.pow(18)), // $1
            ("BTC", 30_000 * 10u128.pow(18)), // $30,000
        ],
        // new prices
        vec![
            ("USD", 98 * 10u128.pow(16)), // $0.98
        ],
        // mod basket
        vec![
            ("USD", 0), // 0% (remove)
            ("BTC", 10u128.pow(18)), // 100%
        ],
        //expected basket
        vec![
            ("BTC", 10u128.pow(18)), // 100%
        ],
        // target
        10u128.pow(18), // $1
        // expected initial
        10u128.pow(18), // $1
        // expected final
        98 * 10u128.pow(16), // $0.98
        10u128.pow(10), // .000001% error
    ),
    mod_index_test_1: (
        "AnIndex",
        // basket
        vec![
            ("USD", 25 * 10u128.pow(16)), // 25%
            ("BTC", 30 * 10u128.pow(16)), // 30%
            ("ETH", 45 * 10u128.pow(16)), // 45%
        ],
        // prices
        vec![
            ("USD", 10u128.pow(18)), // $1
            ("ATOM", 30 * 10u128.pow(18)), // $30
            ("BTC", 30_000 * 10u128.pow(18)), // $30,000
            ("ETH", 2_000 * 10u128.pow(18)), // $2,000
        ],
        // new prices
        vec![
            ("USD", 0_03 * 10u128.pow(16)), // $0.03
            ("BTC", 45_000 * 10u128.pow(18)), // $0.98
            ("ETH", 3_000 * 10u128.pow(18)), // $0.98
        ],
        // mod basket
        vec![
            ("USD", 0), // 0% (remove)
            ("BTC", 10 * 10u128.pow(16)), // decrease to 10%
            ("ATOM", 45 * 10u128.pow(16)), // add at 45%
        ],
        //expected basket
        vec![
            ("BTC", 10u128.pow(17)), // 10%
            ("ATOM", 45 * 10u128.pow(16)), // 45%
            ("ETH", 45 * 10u128.pow(16)), // 45%
        ],
        // target
        10 * 10u128.pow(18), // $10
        // expected initial
        10 * 10u128.pow(18), // $10
        // expected final
        11_325 * 10u128.pow(15), // $11.325
        10u128.pow(10), // .000001% error
    ),
}
