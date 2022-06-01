use crate::{
    common::{Contract, ResponseStatus},
};
use cosmwasm_std::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::utils::Query;

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub owner: HumanAddr,
    pub default_oracle: Contract,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum RegistryOperation {
    Remove { key: String },
    Replace { oracle: Contract, key: String },
    Add { oracle: Contract, key: String },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum HandleMsg {
    UpdateConfig { owner: Option<HumanAddr>, default_oracle: Option<Contract> },
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
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: HumanAddr,
    pub default_oracle: Contract,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OracleResponse {
    pub key: String,
    pub oracle: Contract,
}

pub mod querier {
    use crate::common::*;
    use cosmwasm_std::{to_binary, Querier, QueryRequest, StdResult, WasmQuery};

    use super::QueryMsg;
    use super::*;

    /// Gets the price returned by the oracle stored at key
    pub fn query_price(
        contract: &Contract,
        querier: &impl Querier,
        key: String,
    ) -> StdResult<OraclePrice> {
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.address.clone(),
            callback_code_hash: contract.code_hash.clone(),
            msg: to_binary(&QueryMsg::GetPrice { key })?,
        }))
    }

    /// Gets the prices returned by the oracles stored at that key
    pub fn query_prices(
        contract: &Contract,
        querier: &impl Querier,
        keys: Vec<String>,
    ) -> StdResult<Vec<OraclePrice>> {
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.address.clone(),
            callback_code_hash: contract.code_hash.clone(),
            msg: to_binary(&QueryMsg::GetPrices { keys })?,
        }))
    }

    // Gets the oracle contract stored at key
    pub fn query_oracle(
        contract: &Contract,
        querier: &impl Querier,
        key: String,
    ) -> StdResult<Contract> {
        let resp: OracleResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.address.clone(),
            callback_code_hash: contract.code_hash.clone(),
            msg: to_binary(&QueryMsg::GetOracle { key })?,
        }))?;
        Ok(resp.oracle)
    }
}
