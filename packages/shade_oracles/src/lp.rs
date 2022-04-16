use mulberry_utils::{
    common::types::{Contract, ResponseStatus},
    scrt::*,
    scrt_math::Uint256,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod secretswap {
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
        pub owner: String,
        pub oracle0: Contract,
        pub oracle1: Contract,
        pub factory: Contract,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        UpdateConfig {
            owner: Option<String>,
            oracle0: Option<Contract>,
            oracle1: Option<Contract>,
            factory: Option<Contract>,
        },
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleAnswer {
        UpdateConfig { status: ResponseStatus },
    }

    // We define a custom struct for each query response
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct ConfigResponse {
        pub owner: String,
        pub oracle1: Contract,
        pub oracle2: Contract,
        pub factory: Contract,
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
        pub owner: String,
        pub oracle0: Contract,
        pub oracle1: Contract,
        pub factory: Contract,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        UpdateConfig {
            owner: Option<String>,
            oracle0: Option<Contract>,
            oracle1: Option<Contract>,
            factory: Option<Contract>,
        },
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleAnswer {
        UpdateConfig { status: ResponseStatus },
    }

    // We define a custom struct for each query response
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct ConfigResponse {
        pub owner: String,
        pub oracle1: Contract,
        pub oracle2: Contract,
        pub factory: Contract,
    }
}

pub struct FairLpPriceInfo {
    pub reserve: u128,
    pub price: u128,
    pub decimals: u8,
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
