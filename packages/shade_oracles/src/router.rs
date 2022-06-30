use crate::common::{Contract, ResponseStatus};
use cosmwasm_std::*;
use schemars::JsonSchema;
use secret_toolkit::utils::Query;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub admin_auth: Contract,
    pub default_oracle: Contract,
    pub band: Contract,
    pub quote_symbol: String,
}

#[derive(Serialize, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub address: HumanAddr,
    pub admin_auth: Contract,
    pub default_oracle: Contract,
    pub band: Contract,
    pub quote_symbol: String,
    pub enabled: bool,
}

#[derive(Serialize, Debug, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct UpdateConfig {
    pub admin_auth: Option<Contract>,
    pub default_oracle: Option<Contract>,
    pub band: Option<Contract>,
    pub quote_symbol: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum RegistryOperation {
    Remove { key: String },
    Replace { oracle: Contract, key: String },
    Add { oracle: Contract, key: String },
    UpdateAlias { alias: String, key: String },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum HandleMsg {
    UpdateConfig { config: UpdateConfig },
    UpdateRegistry { operation: RegistryOperation },
    BatchUpdateRegistry { operations: Vec<RegistryOperation> },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum HandleAnswer {
    UpdateConfig { status: ResponseStatus },
    UpdateRegistry { status: ResponseStatus },
    BatchUpdateRegistry { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
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

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OracleResponse {
    pub key: String,
    pub oracle: Contract,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct AdminAuthResponse {
    pub admin_auth: Contract,
}
