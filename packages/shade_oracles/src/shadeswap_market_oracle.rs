use crate::common::{
    Contract,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub router: Contract,
    pub pair: Contract,
    pub symbol: String,
    pub base_peg: String,
    pub only_band: bool,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub router: Contract,
    pub pair: Contract,
    pub symbol: String,
    pub only_band: bool,
    pub base_peg: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    UpdateConfig {
        router: Option<Contract>,
        enabled: Option<bool>,
        only_band: Option<bool>,
    },
}