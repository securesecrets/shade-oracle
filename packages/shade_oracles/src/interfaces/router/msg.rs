use super::registry::{Config, ProtectedKeyInfo, RegistryOperation, UpdateConfig};
use crate::{
    common::status::ContractStatus,
    impl_msg_callbacks,
    interfaces::common::{PriceResponse, PricesResponse},
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint256;
use shade_protocol::{utils::asset::RawContract, Contract};

impl_msg_callbacks!();

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: RawContract,
    pub band: RawContract,
    pub quote_symbol: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetStatus(ContractStatus),
    UpdateProtectedKeys(Vec<(String, Uint256)>),
    UpdateConfig(UpdateConfig),
    UpdateRegistry(RegistryOperation),
    BatchUpdateRegistry(Vec<RegistryOperation>),
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
    #[returns(OraclesResponse)]
    GetOracles { keys: Vec<String> },
    #[returns(PricesResponse)]
    GetPrices { keys: Vec<String> },
    #[returns(KeysResponse)]
    GetKeys {},
    #[returns(ProtectedKeysResponse)]
    GetProtectedKeys {},
}

#[cw_serde]
pub struct OraclesResponse {
    pub oracles: Vec<OracleResponse>,
}

#[cw_serde]
pub struct KeysResponse {
    pub keys: Vec<String>,
}

#[cw_serde]
pub struct ProtectedKeysResponse {
    keys: Vec<ProtectedKeyInfo>,
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
