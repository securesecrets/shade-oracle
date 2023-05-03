use better_secret_math::U256;
use cosmwasm_std::Decimal256;
use std::str::FromStr;

use crate::interfaces::{common::OraclePrice, providers::ReferenceData};

pub struct PricesFixture;

impl PricesFixture {
    pub const USD: &'static str = "USD";
    pub const SCRT: &'static str = "SCRT";
    pub const SILK: &'static str = "SILK";
    pub const BTC: &'static str = "BTC";
    pub const ETH: &'static str = "ETH";
    pub const XAU: &'static str = "XAU";
    pub const FRAX: &'static str = "FRAX";
    pub const OSMO: &'static str = "OSMO";
    pub const USDC: &'static str = "USDC";
    pub const SHD: &'static str = "SHD";

    /// Two prices - USD, SILK
    pub fn basic_prices_1() -> Vec<(&'static str, u128)> {
        vec![
            (Self::USD, 10u128.pow(18)),         // $1
            (Self::SILK, 1_05 * 10u128.pow(16)), // $1.05
        ]
    }

    /// Nine prices - USD, BTC, ETH, XAU, FRAX, OSMO, USDC, SHD, SCRT
    pub fn basic_prices_2() -> Vec<(&'static str, u128)> {
        vec![
            (Self::USD, 1_00 * 10u128.pow(16)),      // $1
            (Self::BTC, 2_939_820 * 10u128.pow(14)), // $29398.2
            (Self::ETH, 183_126 * 10u128.pow(14)),  // $1831.26
            (Self::XAU, 185_265 * 10u128.pow(14)),  // $1852.65
            (Self::FRAX, 1_00 * 10u128.pow(16)),     // $1
            (Self::OSMO, 0_944 * 10u128.pow(16)),    // .944
            (Self::USDC, 1_00 * 10u128.pow(16)),     // $1
            (Self::SHD, 8_00 * 10u128.pow(16)),      // $8
            (Self::SCRT, 11_00 * 10u128.pow(16)),
        ]
    }
}

pub fn generate_price_feed(items: Vec<(&str, &str, u64)>) -> Vec<OraclePrice> {
    let mut prices = vec![];
    for item in items {
        prices.push(OraclePrice::new(item.0.into(), price_data(item.1, item.2)));
    }
    prices
}

pub fn price_data(price: &str, last_updated: u64) -> ReferenceData {
    let price: U256 = Decimal256::from_str(price).unwrap().into();
    ReferenceData {
        rate: price.into(),
        last_updated_base: last_updated,
        last_updated_quote: last_updated,
    }
}
