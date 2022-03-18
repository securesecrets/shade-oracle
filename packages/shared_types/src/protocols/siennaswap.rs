use fadroma::platform::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::asset::Contract;

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SiennaSwapPairInfo {
    pub liquidity_token: Contract,
    pub factory: Contract,
    pub pair: [SiennaDexTokenType; 2],
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
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
