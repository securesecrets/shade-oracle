use cosmwasm_schema::cw_serde;

use shade_oracles::impl_msg_callbacks;
use shade_toolkit::{Contract, RawContract};

impl_msg_callbacks!();

#[cw_serde]
pub struct Config {
    pub router: Contract,
    pub dshd: Contract,
    pub admin_auth: Contract,
    pub enabled: bool,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub router: RawContract,
    pub dshd: RawContract,
    pub admin_auth: RawContract,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        router: Option<RawContract>,
        dshd: Option<RawContract>,
        admin_auth: Option<RawContract>,
        enabled: Option<bool>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetPrice { key: String },
    GetPrices { keys: Vec<String> },
    GetConfig {},
}
