use crate::common::{
    Contract, ResponseStatus,
};
use std::collections::HashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{
    Uint128, HumanAddr,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub admins: Option<Vec<HumanAddr>>,
    pub router: Contract,
    pub symbol: String,
    pub basket: Vec<(String, Uint128)>, //HashMap<String, Uint128>,
    pub target: Uint128,
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
        admins: Option<Vec<HumanAddr>>,
        router: Option<Contract>,
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
        symbol: String,
    },
    GetPrices {
        symbols: Vec<String>,
    },
    GetConfig { },
    Basket { },
    Constants { },
}
