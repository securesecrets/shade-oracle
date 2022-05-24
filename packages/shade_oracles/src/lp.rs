use crate::{common::Contract};
use cosmwasm_std::*;
use fadroma::Uint256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod secretswap {
    use super::*;
    /// Symbol_0 - symbol for asset 0 to be called on oracle router
    ///
    /// Symbol_1 - symbol for asset 1 to be called on oracle router
    ///
    /// Factory - contract that mints the LP token for asset 0 & asset 1
    /// (SecretSwap - Pair | SiennaSwap - Exchange)
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct InitMsg {
        pub owner: HumanAddr,
        pub supported_symbol: String,
        pub symbol_0: String,
        pub symbol_1: String,
        pub router: Contract,
        pub factory: Contract,
    }

    // We define a custom struct for each query response
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct ConfigResponse {
        pub owner: HumanAddr,
        pub factory: Contract,
        pub supported_symbol: String,
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
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct InitMsg {
        pub owner: HumanAddr,
        pub supported_symbol: String,
        pub symbol_0: String,
        pub symbol_1: String,
        pub router: Contract,
        pub factory: Contract,
    }

    // We define a custom struct for each query response
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct ConfigResponse {
        pub owner: HumanAddr,
        pub supported_symbol: String,
        pub symbol_0: String,
        pub symbol_1: String,
        pub router: Contract,
        pub factory: Contract,
        pub enabled: bool,
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum SiennaSwapQueryResponse {
        PairInfo {
            liquidity_token: Contract,
            factory: Contract,
            pair: SiennaSwapPair,
            amount_0: Uint128,
            amount_1: Uint128,
            total_liquidity: Uint128,
            contract_version: u32,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct SiennaSwapPair {
        pub token_0: SiennaDexTokenType,
        pub token_1: SiennaDexTokenType,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum SiennaSwapExchangeQueryMsg {
        PairInfo,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum SiennaDexTokenType {
        CustomToken {
            contract_addr: String,
            token_code_hash: String,
        },
        NativeToken {
            denom: String,
        },
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
) -> StdResult<u128> {
    let normalized_reserve1 = Uint256::from(a.reserve * 10u128.pow((18 - a.decimals).into()));
    let normalized_reserve2 = Uint256::from(b.reserve * 10u128.pow((18 - b.decimals).into()));
    let normalized_supply =
        Uint256::from(total_supply * 10u128.pow((18 - lp_token_decimals).into()));
    let safe_price_a = Uint256::from(a.price);
    let safe_price_b = Uint256::from(b.price);
    let total_value_a = normalized_reserve1.checked_mul(safe_price_a)?;
    let total_value_b = normalized_reserve2.checked_mul(safe_price_b)?;
    let lp_total_value = total_value_a.checked_add(total_value_b)?;
    lp_total_value.checked_div(normalized_supply)?.clamp_u128()
}

/// Calculates the price of an LP token based on https://blog.alphafinance.io/fair-lp-token-pricing/.
///
/// Assumes token prices are normalized to 10^18.
pub fn get_fair_lp_token_price(
    a: FairLpPriceInfo,
    b: FairLpPriceInfo,
    total_supply: u128,
    lp_token_decimals: u8,
) -> StdResult<u128> {
    let normalized_reserve1 = Uint256::from(a.reserve * 10u128.pow((18 - a.decimals).into()));
    let normalized_reserve2 = Uint256::from(b.reserve * 10u128.pow((18 - b.decimals).into()));
    let normalized_supply =
        Uint256::from(total_supply * 10u128.pow((18 - lp_token_decimals).into()));
    let r = normalized_reserve1
        .checked_mul(normalized_reserve2)?
        .sqrt()?;
    let safe_price_a = Uint256::from(a.price);
    let safe_price_b = Uint256::from(b.price);
    let p = safe_price_a.checked_mul(safe_price_b)?.sqrt()?;
    let x = r
        .checked_mul(p)?
        .checked_div(normalized_supply)?
        .checked_mul(Uint256::from(2))?;
    x.clamp_u128()
}
