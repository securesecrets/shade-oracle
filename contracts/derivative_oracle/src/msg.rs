use cosmwasm_schema::cw_serde;

use shade_oracles::impl_msg_callbacks;
use shade_toolkit::{Contract, RawContract};

impl_msg_callbacks!();

#[cw_serde]
pub struct Config {
    pub router: Contract,
    pub derivative: Contract,
    pub admin_auth: Contract,
    pub underlying_key: String,
    pub price_key: String,
    pub rate_key: String,
    pub enabled: bool,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub router: RawContract,
    pub derivative: RawContract,
    pub admin_auth: RawContract,
    pub underlying_key: String,
    pub price_key: String,
    pub rate_key: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        router: Option<RawContract>,
        derivative: Option<RawContract>,
        admin_auth: Option<RawContract>,
        underlying_key: Option<String>,
        price_key: Option<String>,
        rate_key: Option<String>,
        enabled: Option<bool>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetPrice { key: String },
    GetPrices { keys: Vec<String> },
    GetConfig {},
}
