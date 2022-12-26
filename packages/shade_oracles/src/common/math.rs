//! Static classes that help with common math functions.
use super::asset::Asset;
use super::*;
use better_secret_math::{
    core::{bankers_round, exp10, muldiv},
    ud60x18::{mul, sqrt},
    U256,
};

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
            Ok(muldiv(value, exp10(18), exp10(decimals))?)
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

pub struct FairLpPriceInfo {
    pub reserve: u128,
    pub price: u128,
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
    ) -> StdResult<Uint128> {
        let normalized_reserve1: Uint256 =
            TokenMath::normalize_value(a.reserve, a.decimals)?.into();
        let normalized_reserve2: Uint256 =
            TokenMath::normalize_value(b.reserve, b.decimals)?.into();
        let normalized_supply =
            Uint256::from(total_supply * 10u128.pow((18 - lp_token_decimals).into()));
        let safe_price_a = Uint256::from(a.price);
        let safe_price_b = Uint256::from(b.price);
        let total_value_a = normalized_reserve1.checked_mul(safe_price_a)?;
        let total_value_b = normalized_reserve2.checked_mul(safe_price_b)?;
        let lp_total_value = total_value_a.checked_add(total_value_b)?;
        Ok(lp_total_value.checked_div(normalized_supply)?.try_into()?)
    }

    /// Calculates the price of an LP token based on https://blog.alphafinance.io/fair-lp-token-pricing/.
    ///
    /// Assumes token prices are normalized to 10^18.
    pub fn get_fair_lp_token_price(
        a: FairLpPriceInfo,
        b: FairLpPriceInfo,
        total_supply: u128,
        lp_token_decimals: u8,
    ) -> StdResult<Uint128> {
        let normalized_reserve1 = TokenMath::normalize_value(a.reserve, a.decimals)?;
        let normalized_reserve2 = TokenMath::normalize_value(b.reserve, b.decimals)?;
        let normalized_supply = TokenMath::normalize_value(total_supply, lp_token_decimals)?;
        let r = sqrt(mul(normalized_reserve1, normalized_reserve2)?)?;
        let safe_price_a = U256::from(a.price);
        let safe_price_b = U256::from(b.price);
        let p = sqrt(mul(safe_price_a, safe_price_b)?)?;
        Ok(mul(U256::from(2u128), muldiv(r, p, normalized_supply)?)?.into())
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    #[test]
    fn test_get_fair_lp_token_price_1() {
        // Hardcoded data from USDC.e - WAVAX pool on Trader Joe
        // https://analytics.traderjoexyz.com/pairs/0xa389f9430876455c36478deea9769b7ca4e3ddb1
        // https://snowtrace.io/token/0xa389f9430876455c36478deea9769b7ca4e3ddb1#readContract
        let price_info_1 = FairLpPriceInfo {
            reserve: 108206619407191,
            price: 10u128.pow(18),
            decimals: 6,
        };

        let price_info_2 = FairLpPriceInfo {
            reserve: 1234341142020098415888219,
            price: 87 * 10u128.pow(18),
            decimals: 18,
        };

        let total_supply = 9856618006458893260;

        let fair_lp_token_price = LiquidityPoolMath::get_fair_lp_token_price(
            price_info_1,
            price_info_2,
            total_supply,
            18,
        );
        assert_eq!(
            Uint128::from(21872892226771994653562412u128),
            fair_lp_token_price.unwrap()
        )
    }
}