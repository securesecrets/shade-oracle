use crate::{
    common::status::ContractStatus,
    impl_msg_callbacks,
    interfaces::{
        common::{PriceResponse, PricesResponse},
        providers::{Provider, RawProvider},
    },
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint256;
use shade_toolkit::{Contract, RawContract};

use super::registry::ProtectedKeyInfo;

impl_msg_callbacks!();

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: RawContract,
    pub provider: RawProvider,
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
pub struct Config {
    pub this: Contract,
    pub admin_auth: Contract,
    pub provider: Provider,
    pub quote_symbol: String,
}

#[cw_serde]
pub enum RegistryOperation {
    RemoveKeys {
        keys: Vec<String>,
    },
    SetKeys {
        oracle: RawContract,
        keys: Vec<String>,
    },
    SetProtection {
        infos: Vec<ProtectedKeyInfo>,
    },
    RemoveProtection {
        keys: Vec<String>,
    },
}

#[cw_serde]
pub struct UpdateConfig {
    pub admin_auth: Option<RawContract>,
    pub provider: Option<RawProvider>,
    pub quote_symbol: Option<String>,
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

pub type OraclesResponse = Vec<OracleResponse>;
pub type KeysResponse = Vec<String>;
pub type ProtectedKeysResponse = Vec<ProtectedKeyInfo>;

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
