use cosmwasm_schema::cw_serde;
use shade_protocol::utils::asset::{UnvalidatedContract, Contract};
use shade_protocol::utils::{ExecuteCallback, InstantiateCallback};

use crate::BLOCK_SIZE;

#[cw_serde]
pub struct Config {
    pub router: Contract,
    pub pair: Contract,
    pub symbol: String,
    pub base_peg: String,
    pub only_band: bool,
    pub enabled: bool,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub router: Contract,
    pub pair: Contract,
    pub symbol: String,
    pub only_band: bool,
    pub base_peg: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    UpdateConfig {
        router: Option<Contract>,
        enabled: Option<bool>,
        only_band: Option<bool>,
    },
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}