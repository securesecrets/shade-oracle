use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use mulberry_utils::{
    common::types::{Contract, ResponseStatus},
    scrt::*,
};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub owner: String,
    pub deposit_token_oracle: Contract,
    pub strategy: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        owner: Option<String>,
        deposit_token_oracle: Option<Contract>,
        strategy: Option<Contract>,
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
    pub deposit_token_oracle: Contract,
    pub deposit_token: Contract,
    pub share_token: Contract,
    pub strategy: Contract,
}