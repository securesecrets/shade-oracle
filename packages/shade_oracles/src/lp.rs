use crate::common::{normalize_price, sqrt, Contract};
use cosmwasm_std::{Uint128, Uint256};
use cosmwasm_std::*;
use cosmwasm_schema::cw_serde;

pub mod secretswap {
    use super::*;
    /// Symbol_0 - symbol for asset 0 to be called on oracle router
    ///
    /// Symbol_1 - symbol for asset 1 to be called on oracle router
    ///
    /// Factory - contract that mints the LP token for asset 0 & asset 1
    /// (SecretSwap - Pair | SiennaSwap - Exchange)
    #[cw_serde]
    pub struct InstantiateMsg {
        pub supported_key: String,
        pub symbol_0: String,
        pub symbol_1: String,
        pub router: Contract,
        pub pair: Contract,
    }

    // We define a custom struct for each query response
    #[cw_serde]
    pub struct Config {
        pub pair: Contract,
        pub supported_key: String,
        pub symbol_0: String,
        pub symbol_1: String,
        pub router: Contract,
        pub enabled: bool,
    }
}

pub mod siennaswap {
    use super::*;
    /// Oracle1 - contract for an oracle of asset 1
    ///
    /// Oracle2 - contract for an oracle of asset 2
    ///
    /// Factory - contract that mints the LP token for asset 1 & asset 2
    /// (SecretSwap - Pair | SiennaSwap - Exchange)
    #[cw_serde]
    pub struct InstantiateMsg {
        pub supported_key: String,
        pub symbol_0: String,
        pub symbol_1: String,
        pub router: Contract,
        pub exchange: Contract,
    }

    // We define a custom struct for each query response
    #[cw_serde]
    pub struct Config {
        pub supported_key: String,
        pub symbol_0: String,
        pub symbol_1: String,
        pub router: Contract,
        pub exchange: Contract,
        pub enabled: bool,
    }

    #[cw_serde]
    pub struct PairData {
        pub lp_token: Contract,
        pub token0_decimals: u8,
        pub token1_decimals: u8,
    }
}

pub struct FairLpPriceInfo {
    pub reserve: u128,
    pub price: u128,
    pub decimals: u8,
}

/// Calculates the spot price of an LP token
pub fn get_lp_token_spot_price(
    a: FairLpPriceInfo,
    b: FairLpPriceInfo,
    total_supply: u128,
    lp_token_decimals: u8,
) -> StdResult<Uint128> {
    let normalized_reserve1 =
        Uint256::from_uint128(normalize_price(Uint128::from(a.reserve), a.decimals));
    let normalized_reserve2 = Uint256::from(normalize_price(Uint128::from(b.reserve), b.decimals));
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
    let normalized_reserve1 = Uint256::from(normalize_price(Uint128::from(a.reserve), a.decimals));
    let normalized_reserve2 = Uint256::from(normalize_price(Uint128::from(b.reserve), b.decimals));
    let normalized_supply =
        Uint256::from(total_supply * 10u128.pow((18 - lp_token_decimals).into()));
    let r = sqrt(normalized_reserve1.checked_mul(normalized_reserve2)?)?;
    let safe_price_a = Uint256::from(a.price);
    let safe_price_b = Uint256::from(b.price);
    let p = sqrt(safe_price_a.checked_mul(safe_price_b)?)?;
    Ok(r.checked_mul(p)?
        .checked_div(normalized_supply)?
        .checked_mul(Uint256::from(2u128))?
        .try_into()?)
}
