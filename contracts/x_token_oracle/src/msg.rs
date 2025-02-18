use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal256;

use cosmwasm_std::Addr;
use shade_oracles::impl_msg_callbacks;
use shade_toolkit::{Contract, RawContract};

impl_msg_callbacks!();

#[cw_serde]
pub struct Config {
    pub router: Contract,
    pub admin_auth: Contract,
    pub money_market: Contract,
    pub enabled: bool,
}

#[cw_serde]
pub struct KeyConfig {
    pub key: String,
    pub vault: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub router: RawContract,
    pub admin_auth: RawContract,
    pub money_market: RawContract,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        router: Option<RawContract>,
        admin_auth: Option<RawContract>,
        money_market: Option<RawContract>,
        enabled: Option<bool>,
    },
    RegisterVault(KeyConfig),
    RegisterVaults(Vec<KeyConfig>),
}

#[cw_serde]
pub enum QueryMsg {
    GetPrice { key: String },
    GetPrices { keys: Vec<String> },
    GetConfig {},
}
