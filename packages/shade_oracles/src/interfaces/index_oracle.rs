use crate::{Contract, ResponseStatus, BLOCK_SIZE};
use crate::{InstantiateCallback, ExecuteCallback};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use shade_protocol::utils::Query;


#[cw_serde]
pub struct Config {
    pub router: Contract,
    pub only_band: bool,
    pub enabled: bool,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub router: Contract,
    pub symbol: String,
    pub basket: Vec<(String, Uint128)>, //HashMap<String, Uint128>,
    pub target: Uint128,
    pub only_band: bool,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum ExecuteMsg {
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

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum HandleAnswer {
    ModBasket { status: ResponseStatus },
    UpdateConfig { status: ResponseStatus },
}

#[cw_serde]
pub enum QueryMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    GetPrice { key: String },
    GetPrices { keys: Vec<String> },
    GetConfig {},
    Basket {},
    //Constants { },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum QueryAnswer {
    Basket {
        basket: Vec<(String, Uint128, Uint128)>,
    },
}
