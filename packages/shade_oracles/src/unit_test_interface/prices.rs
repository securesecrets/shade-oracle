use better_secret_math::U256;
use cosmwasm_std::{Decimal256, Uint128};
use std::str::FromStr;

use crate::{interfaces::band::ReferenceData, interfaces::OraclePrice};

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
        rate: Uint128::new(price.as_u128()),
        last_updated_base: last_updated,
        last_updated_quote: last_updated,
    }
}
