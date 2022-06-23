use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{HumanAddr};

use crate::common::Contract;
use secret_toolkit::utils::Query;

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SiennaSwapPairInfoResponse {
    pub pair_info: SiennaSwapPairInfo,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SiennaSwapPairInfo {
    pub liquidity_token: Contract,
    pub factory: Contract,
    pub pair: Pair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SiennaSwapExchangeQueryMsg {
    PairInfo,
    SwapSimulation { offer: TokenTypeAmount },
}

impl Query for SiennaSwapExchangeQueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SiennaDexTokenType {
    CustomToken {
        contract_addr: HumanAddr,
        token_code_hash: String,
    },
    NativeToken {
        denom: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pair {
    pub token_0: SiennaDexTokenType,
    pub token_1: SiennaDexTokenType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenTypeAmount {
    pub amount: Uint128,
    pub token: SiennaDexTokenType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SwapSimulation {
    pub offer: TokenTypeAmount,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub commission_amount: Uint128,
}
