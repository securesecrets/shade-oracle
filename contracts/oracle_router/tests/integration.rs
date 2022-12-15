#![allow(
    clippy::zero_prefixed_literal,
    clippy::inconsistent_digit_grouping,
    clippy::too_many_arguments
)]

use cosmwasm_std::{Addr, Decimal256, StdResult, Uint128, Uint256, Uint64};
use rstest::*;
use shade_oracles::{
    common::{PriceResponse, PricesResponse},
    core::{
        admin::{self, helpers::AdminPermissions},
        ExecuteCallback, InstantiateCallback, Query,
    },
    interfaces::router::{msg::*, *},
};
use shade_oracles_multi_test::{
    multi::helpers::{AdminAuthHelper, OracleCore},
    multi::router::OracleRouter,
    App, MultiTestable,
};
use std::collections::HashMap;
use std::str::FromStr;

pub fn basic_prices_1() -> Vec<(&'static str, u128)> {
    vec![
        ("USD", 10u128.pow(18)),         // $1
        ("SILK", 1_05 * 10u128.pow(16)), // $1.05
    ]
}

pub fn basic_prices_2() -> Vec<(&'static str, u128)> {
    vec![
        ("USD", 1_00 * 10u128.pow(16)),      // $1
        ("BTC", 29_398_20 * 10u128.pow(14)), // $29398.2
        ("ETH", 1_831_26 * 10u128.pow(14)),  // $1831.26
        ("XAU", 1_852_65 * 10u128.pow(14)),  // $1852.65
    ]
}

#[test]
fn protected_query_admin_tests() {
    let mut keys = vec![];
    let prices = vec![("USD", 1_00 * 10u128.pow(16))];
    let prices: HashMap<String, Uint128> = prices
        .into_iter()
        .map(|(sym, p)| {
            keys.push(sym.to_string());
            (sym.to_string(), p.into())
        })
        .collect();
    let test_prices = prices.clone();
    let user = Addr::unchecked("superadmin");
    let mut app = &mut App::default();

    let oracle_core = OracleCore::setup(app, &user, prices, None, None, None).unwrap();
    let band = oracle_core.band;
    let admin_auth = AdminAuthHelper(oracle_core.admin_auth.clone());
    let router = oracle_core.router;

    let mock_user = Addr::unchecked("randomuser");
}

#[rstest]
fn protected_query_tests() {
    let mut keys = vec![];
    let prices = vec![("USD", 1_00 * 10u128.pow(16))];
    let prices: HashMap<String, Uint128> = prices
        .into_iter()
        .map(|(sym, p)| {
            keys.push(sym.to_string());
            (sym.to_string(), p.into())
        })
        .collect();
    let test_prices = prices.clone();
    let user = Addr::unchecked("superadmin");
    let mut app = &mut App::default();

    let oracle_core = OracleCore::setup(app, &user, prices, None, None, None).unwrap();
    let band = oracle_core.band;
    let admin_auth = AdminAuthHelper(oracle_core.admin_auth.clone());
    let router = oracle_core.router;
    router
        .protect_key(
            &user,
            app,
            "USD".to_string(),
            Decimal256::percent(4),
            Uint256::from_u128(1_00 * 10u128.pow(16)),
        )
        .unwrap();
    assert!(router.query_price(app, "USD".to_string()).is_ok());
    assert!(router.query_prices(app, vec!["USD".to_string()]).is_ok());
    let prices = vec![("USD", 1_05 * 10u128.pow(16))];
    let prices: HashMap<String, Uint128> = prices
        .into_iter()
        .map(|(sym, p)| {
            keys.push(sym.to_string());
            (sym.to_string(), p.into())
        })
        .collect();
    band.update_prices(&user, app, prices, app.block_info().time.seconds());
    //let resp = router.query_price(app, "USD".to_string());
    assert!(router.query_price(app, "USD".to_string()).is_err());
    assert!(router.query_prices(app, vec!["USD".to_string()]).is_err());

    let bot = Addr::unchecked("bot");
    let resp = router.update_protected_keys(
        &bot,
        app,
        vec![("USD".to_string(), Uint256::from_u128(1_05 * 10u128.pow(16)))],
    );
    assert!(resp.is_err());

    admin_auth.register_admin(&user, app, bot.to_string());
    admin_auth.grant_access(
        &user,
        app,
        bot.to_string(),
        vec![AdminPermissions::OraclesPriceBot.into_string()],
    );

    let resp = router.update_protected_keys(
        &bot,
        app,
        vec![("USD".to_string(), Uint256::from_u128(1_05 * 10u128.pow(16)))],
    );
    assert!(resp.is_ok());
    assert!(router.query_price(app, "USD".to_string()).is_ok());
    assert!(router.query_prices(app, vec!["USD".to_string()]).is_ok());
}

#[rstest]
#[case(basic_prices_1())]
#[case(basic_prices_2())]
fn basic_query_test(#[case] prices: Vec<(&str, u128)>) {
    let mut keys = vec![];
    let prices: HashMap<String, Uint128> = prices
        .into_iter()
        .map(|(sym, p)| {
            keys.push(sym.to_string());
            (sym.to_string(), p.into())
        })
        .collect();
    let test_prices = prices.clone();
    let user = Addr::unchecked("superadmin");
    let mut app = App::default();

    let oracle_core = OracleCore::setup(&mut app, &user, prices, None, None, None).unwrap();
    let resp = oracle_core.router.query_prices(&app, keys).unwrap();
    for price in resp.prices {
        assert_eq!(&price.data.rate, test_prices.get(price.key()).unwrap());
    }
}
