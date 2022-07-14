use crate::common::{Contract, ResponseStatus};
use cosmwasm_std::*;
use cosmwasm_schema::cw_serde;
use secret_toolkit::utils::Query;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: Contract,
    pub default_oracle: Contract,
    pub band: Contract,
    pub quote_symbol: String,
}

#[cw_serde]
pub struct Config {
    pub address: Addr,
    pub admin_auth: Contract,
    pub default_oracle: Contract,
    pub band: Contract,
    pub quote_symbol: String,
    pub enabled: bool,
}

#[cw_serde]
pub struct UpdateConfig {
    pub admin_auth: Option<Contract>,
    pub default_oracle: Option<Contract>,
    pub band: Option<Contract>,
    pub quote_symbol: Option<String>,
    pub enabled: Option<bool>,
}

#[cw_serde]
pub enum RegistryOperation {
    Remove { key: String },
    Replace { oracle: Contract, key: String },
    Add { oracle: Contract, key: String },
    UpdateAlias { alias: String, key: String },
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { config: UpdateConfig },
    UpdateRegistry { operation: RegistryOperation },
    BatchUpdateRegistry { operations: Vec<RegistryOperation> },
}

#[cw_serde]
pub enum HandleAnswer {
    UpdateConfig { status: ResponseStatus },
    UpdateRegistry { status: ResponseStatus },
    BatchUpdateRegistry { status: ResponseStatus },
}

#[cw_serde]
pub enum QueryMsg {
    GetConfig {},
    /// Get oracle at that key
    GetOracle {
        key: String,
    },
    /// Get price of oracle at that key
    GetPrice {
        key: String,
    },
    GetOracles {
        keys: Vec<String>,
    },
    GetPrices {
        keys: Vec<String>,
    },
    GetAdminAuth {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct OracleResponse {
    pub key: String,
    pub oracle: Contract,
}

#[cw_serde]
pub struct AdminAuthResponse {
    pub admin_auth: Contract,
}
