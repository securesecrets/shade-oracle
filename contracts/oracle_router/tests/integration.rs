#![allow(
    clippy::zero_prefixed_literal,
    clippy::inconsistent_digit_grouping,
    clippy::too_many_arguments
)]

use cosmwasm_std::{Addr, Decimal256, Uint128, Uint64};
use rstest::*;
use shade_oracles::{
    common::{PriceResponse, PricesResponse},
    core::{ExecuteCallback, InstantiateCallback, Query},
    interfaces::router::{msg::*, *},
};
use shade_oracles_multi_test::{
    multi::helpers::{OracleCore, OracleDeps},
    multi::router::OracleRouter,
    App, MultiTestable,
};
use std::collections::HashMap;
use std::str::FromStr;

#[rstest]
#[case(
        vec![
            ("USD", 10u128.pow(18)), // $1
            ("SILK", 1_05 * 10u128.pow(16)), // $1.05
        ],
)]
#[case(
    vec![
        ("USD", 1_00 * 10u128.pow(14)), // $1
        ("BTC", 29_398_20 * 10u128.pow(14)), // $29398.2
        ("ETH", 1_831_26 * 10u128.pow(14)), // $1831.26
        ("XAU", 1_852_65 * 10u128.pow(14)), // $1852.65
    ],
)]
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

    let oracle_core = OracleCore::setup(&mut app, &user, prices, None, None, None, None).unwrap();
    let router = oracle_core.get(OracleDeps::OracleRouter);

    let resp: PricesResponse = QueryMsg::GetPrices { keys }
        .test_query(&router, &app)
        .unwrap();
    for price in resp.prices {
        assert_eq!(&price.data.rate, test_prices.get(price.key()).unwrap());
    }
}
