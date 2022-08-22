use super::registry::{Config, RegistryOperation, UpdateConfig};
use crate::common::{PriceResponse, PricesResponse};
use crate::mulberry::impl_msg_callbacks;
use cosmwasm_schema::{cw_serde, QueryResponses};
use mulberry::common::ContractStatus;
use shade_protocol::Contract;

impl_msg_callbacks!();

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: Contract,
    pub default_oracle: Contract,
    pub band: Contract,
    pub quote_symbol: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetStatus { status: ContractStatus },
    UpdateConfig { config: UpdateConfig },
    UpdateRegistry { operation: RegistryOperation },
    BatchUpdateRegistry { operations: Vec<RegistryOperation> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    GetConfig {},
    /// Get oracle at that key
    #[returns(OracleResponse)]
    GetOracle { key: String },
    /// Get price of oracle at that key
    #[returns(PriceResponse)]
    GetPrice { key: String },
    #[returns(Vec<OracleResponse>)]
    GetOracles { keys: Vec<String> },
    #[returns(PricesResponse)]
    GetPrices { keys: Vec<String> },
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
    pub status: ContractStatus,
}

#[cw_serde]
pub struct OracleResponse {
    pub key: String,
    pub oracle: Contract,
}
