use crate::common::{
    Contract, ResponseStatus,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{
    Uint128, HumanAddr,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admins: Vec<HumanAddr>,
    pub router: Contract,
    pub pair: Contract,
    pub symbol: String,
    pub base_peg: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub admins: Option<Vec<HumanAddr>>,
    pub router: Contract,
    pub pair: Contract,
    pub symbol: String,
    pub base_peg: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    UpdateConfig {
        admins: Option<Vec<HumanAddr>>,
        router: Option<Contract>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateConfig {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetPrice {
        key: String,
    },
    GetPrices {
        keys: Vec<String>,
    },
    GetConfig { },
}

/*
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Basket {
        basket: Vec<(String, Uint128, Uint128)>,
    },
}
*/
