use crate::common::{
    Contract, ResponseStatus,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_math_compat::Uint128;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub router: Contract,
    pub only_band: bool,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub router: Contract,
    pub symbol: String,
    pub basket: Vec<(String, Uint128)>, //HashMap<String, Uint128>,
    pub target: Uint128,
    pub only_band: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    ModBasket {
        basket: Vec<(String, Uint128)>,
    },
    UpdateConfig {
        router: Option<Contract>,
        enabled: Option<bool>,
        only_band: Option<bool>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    ModBasket {
        status: ResponseStatus,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    GetPrice {
        key: String,
    },
    GetPrices {
        keys: Vec<String>,
    },
    GetConfig { },
    Basket { },
    //Constants { },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Basket {
        basket: Vec<(String, Uint128, Uint128)>,
    },
}
