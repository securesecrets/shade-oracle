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
    // base asset to apply multiplier to
    pub base_key: String,
    pub multiplier: Decimal256,
    pub price_key: String,
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
<<<<<<<< HEAD:contracts/x_token_oracle/src/msg.rs
    pub money_market: RawContract,
========
    pub base_key: String,
    pub multiplier: Decimal256,
    pub price_key: String,
>>>>>>>> origin/develop:contracts/multiplier_oracle/src/msg.rs
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        router: Option<RawContract>,
        admin_auth: Option<RawContract>,
        money_market: Option<RawContract>,
        enabled: Option<bool>,
        multiplier: Option<Decimal256>,
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
