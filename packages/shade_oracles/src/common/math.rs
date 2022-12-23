use better_secret_math::{
    core::{bankers_round, exp10, muldiv},
    U256,
};
use cosmwasm_std::StdResult;

use super::asset::Asset;

pub struct TokenMath;

impl TokenMath {
    /// Accepts the value and the decimal precision of that value.
    ///
    /// Can handle values that have more than 18 decimal places.
    /// Normalizes to 18 decimal places.
    pub fn normalize_value(value: impl Into<U256>, decimals: u8) -> StdResult<U256> {
        let value = value.into();
        if decimals == 18 {
            Ok(value)
        } else {
            Ok(muldiv(value, exp10(18), exp10(decimals))?.into())
        }
    }
    /// Normalizes the asset amount from being based off asset decimals -> 18 decimals.
    pub fn normalize_asset_amount(asset: &Asset, amount: impl Into<U256>) -> StdResult<U256> {
        if asset.decimals == 18 {
            Ok(amount.into())
        } else {
            muldiv(amount.into(), exp10(18), exp10(asset.decimals))
        }
    }
    /// Gets the amount of asset the amount normalized to 18 decimals represents.
    /// Applies bankers rounding to reduce token amount loss.
    pub fn get_asset_amount(asset: &Asset, normalized_amount: impl Into<U256>) -> StdResult<U256> {
        if asset.decimals == 18 {
            Ok(normalized_amount.into())
        } else {
            let precision_diff = 18 - asset.decimals;
            let amount =
                bankers_round(normalized_amount.into(), precision_diff) / exp10(precision_diff);
            Ok(amount)
        }
    }
}
