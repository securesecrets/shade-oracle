use fadroma::{
    scrt::ContractLink,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128};

use secret_toolkit::utils::Query;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenAmount<A> {
    pub token: TokenType<A>,
    pub amount: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType<A> {
    CustomToken {
        contract_addr: A,
        token_code_hash: String,
    },
    NativeToken {
        denom: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenPair<A>(pub TokenType<A>, pub TokenType<A>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenPairAmount<A: Clone> {
    pub pair:     TokenPair<A>,
    pub amount_0: Uint128,
    pub amount_1: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ShadeSwapQueryMsg {
    GetPairInfo,
    GetEstimatedPrice { offer: TokenAmount<HumanAddr> },
}

impl Query for ShadeSwapQueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PairInfoResponse {
    pub liquidity_token: ContractLink<HumanAddr>,
    pub factory: ContractLink<HumanAddr>,
    pub pair: TokenPair<HumanAddr>,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EstimatedPriceResponse {
    pub estimated_price: Uint128
}
