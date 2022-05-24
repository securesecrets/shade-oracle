use crate::{common::Contract};
use cosmwasm_std::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub owner: String,
    pub deposit_token_oracle: Contract,
    pub strategy: Contract,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: String,
    pub deposit_token_oracle: Contract,
    pub deposit_token: Contract,
    pub share_token: Contract,
    pub strategy: Contract,
    pub enabled: bool,
}
