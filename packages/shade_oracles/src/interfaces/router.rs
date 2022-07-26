use crate::BLOCK_SIZE;
use cosmwasm_std::*;
use cosmwasm_schema::{cw_serde, QueryResponses};
use shade_protocol::{
    utils::generic_response::ResponseStatus,
    utils::{InstantiateCallback, ExecuteCallback, Query},
    Contract,
};
use crate::common::{PriceResponse, PricesResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: Contract,
    pub default_oracle: Contract,
    pub band: Contract,
    pub quote_symbol: String,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
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
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    GetConfig {},
    /// Get oracle at that key
    #[returns(OracleResponse)]
    GetOracle {
        key: String,
    },
    /// Get price of oracle at that key
    #[returns(PriceResponse)]
    GetPrice {
        key: String,
    },
    #[returns(Vec<OracleResponse>)]
    GetOracles {
        keys: Vec<String>,
    },
    #[returns(PricesResponse)]
    GetPrices {
        keys: Vec<String>,
    },
    #[returns(AdminAuthResponse)]
    GetAdminAuth {},
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
