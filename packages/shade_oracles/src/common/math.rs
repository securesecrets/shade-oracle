//! Static classes that help with common math functions.
use super::asset::Asset;
use super::*;
use better_secret_math::{
    common::{bankers_round, exp10, muldiv, abs_diff},
    ud60x18::{mul, sqrt},
    U256,
};
use cosmwasm_std::Decimal256;

pub struct GeneralMath;
impl GeneralMath {
    // Require that a is within some % of b.
    pub fn require_within_precision(a: impl Into<U256> + Copy, b: impl Into<U256> + Copy, tolerance: impl Into<U256> + Copy) -> StdResult<()> {
        let a: U256 = a.into();
        let b: U256 = b.into();
        let tolerance: U256 = tolerance.into();
        let diff = abs_diff(a, b);
        if b == U256::ZERO && a == U256::ZERO {
            Ok(())
        } else {
            let actual_deviation = Decimal256::from_ratio(diff, b);
            let max_deviation = tolerance.into();
            if actual_deviation > max_deviation {
                Err(StdError::generic_err(format!(
                    "Expected {} to be within {} of {}. (Actual deviation: {})",
                    a, tolerance, b, actual_deviation
                )))
            } else {
                Ok(())
            }
        }
    }
}

/// Provides methods that deal with token prices, values, and amounts.
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
            Ok(muldiv(value, exp10(18), exp10(decimals as u16))?)
        }
    }
    /// Normalizes the asset amount from being based off asset decimals -> 18 decimals.
    pub fn normalize_asset_amount(asset: &Asset, amount: impl Into<U256>) -> StdResult<U256> {
        if asset.decimals == 18 {
            Ok(amount.into())
        } else {
            muldiv(amount.into(), exp10(18), exp10(asset.decimals as u16))
        }
    }
    /// Gets the amount of asset the amount normalized to 18 decimals represents.
    /// Applies bankers rounding to reduce token amount loss.
    pub fn get_asset_amount(asset: &Asset, normalized_amount: impl Into<U256>) -> StdResult<U256> {
        if asset.decimals == 18 {
            Ok(normalized_amount.into())
        } else {
            let precision_diff = 18 - asset.decimals;
            let amount = bankers_round(normalized_amount.into(), precision_diff)
                / exp10(precision_diff as u16);
            Ok(amount)
        }
    }
}

pub struct FairLpPriceInfo {
    pub reserve: U256,
    pub price: U256,
    pub decimals: u8,
}
pub struct LiquidityPoolMath;

impl LiquidityPoolMath {
    /// Calculates the spot price of an LP token
    pub fn get_lp_token_spot_price(
        a: FairLpPriceInfo,
        b: FairLpPriceInfo,
        total_supply: u128,
        lp_token_decimals: u8,
    ) -> StdResult<Uint256> {
        let normalized_reserve1: Uint256 =
            TokenMath::normalize_value(a.reserve, a.decimals)?.into();
        let normalized_reserve2: Uint256 =
            TokenMath::normalize_value(b.reserve, b.decimals)?.into();
        let normalized_supply =
            Uint256::from(total_supply * 10u128.pow((18 - lp_token_decimals).into()));
        let total_value_a = normalized_reserve1.checked_mul(a.price.into())?;
        let total_value_b = normalized_reserve2.checked_mul(b.price.into())?;
        let lp_total_value = total_value_a.checked_add(total_value_b)?;
        Ok(lp_total_value.checked_div(normalized_supply)?)
    }

    /// Calculates the price of an LP token based on https://blog.alphafinance.io/fair-lp-token-pricing/.
    ///
    /// Assumes token prices are normalized to 10^18.
    pub fn get_fair_lp_token_price(
        a: FairLpPriceInfo,
        b: FairLpPriceInfo,
        total_supply: u128,
        lp_token_decimals: u8,
    ) -> StdResult<Uint256> {
        let normalized_reserve1 = TokenMath::normalize_value(a.reserve, a.decimals)?;
        let normalized_reserve2 = TokenMath::normalize_value(b.reserve, b.decimals)?;
        let normalized_supply = TokenMath::normalize_value(total_supply, lp_token_decimals)?;
        let r = sqrt(mul(normalized_reserve1, normalized_reserve2)?)?;
        let p = sqrt(mul(a.price, b.price)?)?;
        let rp2 = muldiv(r, p, normalized_supply)? * U256::from(2u128);
        Ok(rp2.into())
    }
}

#[cfg(test)]
pub mod test {
    use better_secret_math::common::abs_diff;
    use cosmwasm_std::Decimal256;

    use super::*;
    #[test]
    fn test_get_fair_lp_token_price_1() {
        // Hardcoded data from USDC.e - WAVAX pool on Trader Joe
        // https://analytics.traderjoexyz.com/pairs/0xa389f9430876455c36478deea9769b7ca4e3ddb1
        // https://snowtrace.io/token/0xa389f9430876455c36478deea9769b7ca4e3ddb1#readContract
        let price_info_1 = FairLpPriceInfo {
            reserve: 108206619407191u128.into(),
            price: 10u128.pow(18).into(),
            decimals: 6,
        };

        let price_info_2 = FairLpPriceInfo {
            reserve: 1234341142020098415888219u128.into(),
            price: U256::new(87u128 * 10u128.pow(18)),
            decimals: 18,
        };

        let total_supply = 9856618006458893260;

        let fair_lp_token_price = LiquidityPoolMath::get_fair_lp_token_price(
            price_info_1,
            price_info_2,
            total_supply,
            18,
        );

        let expected = 21872892226771994653562412u128;
        let diff = abs_diff(
            Uint128::from(expected).into(),
            fair_lp_token_price.unwrap().into(),
        );
        let deviation = Decimal256::from_ratio(diff, Uint256::from_u128(expected));
        assert!(deviation < Decimal256::from_ratio(1u128, 10u128.pow(10)))
    }
}
