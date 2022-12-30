use std::cmp::max;

use super::*;
use crate::BLOCK_SIZE;
use better_secret_math::core::{exp10, muldiv};
use better_secret_math::U256;
use shade_protocol::utils::Query;

pub mod config;

mod error;
use super::band::{BtrReferenceData, ReferenceData};

/// Default Query API for all oracles.
///
/// Every oracle must support these 3 methods in addition to any specific ones it wants to support.
#[cw_serde]
#[derive(QueryResponses)]
pub enum OracleQuery {
    #[returns(PriceResponse)]
    /// Returns PriceResponse
    GetPrice { key: String },
    #[returns(PricesResponse)]
    /// Returns PricesResponse
    GetPrices { keys: Vec<String> },
}

pub type PriceResponse = OraclePrice;
pub type PricesResponse = Vec<OraclePrice>;

impl Query for OracleQuery {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
#[derive(Default)]
pub struct OraclePrice {
    pub key: String,
    pub data: ReferenceData,
}

impl OraclePrice {
    pub fn new(key: String, reference_data: ReferenceData) -> Self {
        OraclePrice {
            key,
            data: reference_data,
        }
    }
    pub fn key(&self) -> &String {
        &self.key
    }
    pub fn data(&self) -> &ReferenceData {
        &self.data
    }
}

/// Variant of OraclePrice that is optimized for math.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct BtrOraclePrice {
    pub key: String,
    pub data: BtrReferenceData,
}

impl From<OraclePrice> for BtrOraclePrice {
    fn from(o: OraclePrice) -> Self {
        BtrOraclePrice {
            key: o.key.clone(),
            data: o.data().clone().into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<OraclePrice> for BtrOraclePrice {
    fn into(self) -> OraclePrice {
        OraclePrice {
            key: self.key.clone(),
            data: self.data.into(),
        }
    }
}

impl BtrOraclePrice {
    pub fn key(&self) -> &String {
        &self.key
    }
    pub fn data(&self) -> &BtrReferenceData {
        &self.data
    }
    pub fn time_since_updated(&self, time: &Timestamp) -> StdResult<u64> {
        let now = time.seconds();
        let base = self.data().last_updated_base;
        let quote = self.data().last_updated_quote;
        let time_since_base = now - base;
        let time_since_quote = now - quote;
        let time_since_updated = max(time_since_base, time_since_quote);
        Ok(time_since_updated)
    }
    /// Allows us to pass a variable amount of precision decimals in the future
    /// in case our oracles lose their constant decimal precision (currently 18).
    /// Gets the value for some amount using the price.
    pub fn calc_value(&self, amount: U256) -> StdResult<U256> {
        let price_precision = exp10(18);
        muldiv(amount, self.data.rate, price_precision)
    }
    /// Gets the amount equivalent to the provided value divided by the unit price.
    pub fn calc_amount(
        &self,
        value: U256,
        value_precision: u8,
        amount_precision: u8,
    ) -> StdResult<U256> {
        let price_precision = exp10(18);
        let value_precision = exp10(value_precision as u16);
        let amount_precision = exp10(amount_precision as u16);

        let normalized_value = muldiv(value, price_precision, value_precision)?;
        muldiv(normalized_value, amount_precision, self.data.rate)
    }

    pub fn is_stale_price(
        &self,
        delay_tolerance: u64,
        current_time: &Timestamp,
    ) -> StdResult<bool> {
        if self.time_since_updated(current_time)?.gt(&delay_tolerance) {
            return Ok(true);
        }
        Ok(false)
    }
}

pub fn throw_unsupported_symbol_error(key: String) -> StdError {
    StdError::generic_err(format!("{} is not supported as a key.", key))
}

pub fn is_disabled(enabled: bool) -> StdResult<()> {
    if !enabled {
        Err(StdError::generic_err("Deprecated oracle."))
    } else {
        Ok(())
    }
}
