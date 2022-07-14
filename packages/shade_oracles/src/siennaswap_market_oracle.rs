use cosmwasm_schema::cw_serde;
use crate::common::Contract;


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
pub struct InitMsg {
    pub router: Contract,
    pub pair: Contract,
    pub symbol: String,
    pub only_band: bool,
    pub base_peg: Option<String>,
}

#[cw_serde]
pub enum HandleMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    UpdateConfig {
        router: Option<Contract>,
        only_band: Option<bool>,
        enabled: Option<bool>,
    },
}
