use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use mulberry_utils::{
    common::types::{Contract, ResponseStatus},
    HandlePaddable,
};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub owner: String,
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
    ChangeOwner { new_owner: String },
    UpdateRegistry { operation: RegistryOperation },
    BatchUpdateRegistry { operations: Vec<RegistryOperation> },
}

impl HandlePaddable for HandleMsg {}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum HandleAnswer {
    ChangeOwner { status: ResponseStatus },
    UpdateRegistry { status: ResponseStatus },
    BatchUpdateRegistry { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum QueryMsg {
    GetOwner {},
    /// Get oracle at that key
    GetOracle {
        key: String,
    },
    /// Get price of oracle at that key
    GetPrice {
        key: String,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OracleResponse {
    pub oracle: Contract,
}

pub mod querier {
    use crate::common::PriceResponse;
    use mulberry_utils::scrt::{
        to_binary, HumanAddr, Querier, QueryRequest, StdResult, WasmQuery,
    };

    use super::QueryMsg;
    use super::*;

    pub fn query_price(
        contract: &Contract,
        querier: &impl Querier,
        key: String,
    ) -> StdResult<PriceResponse> {
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: HumanAddr(contract.address.clone()),
            callback_code_hash: contract.code_hash.clone(),
            msg: to_binary(&QueryMsg::GetPrice { key })?,
        }))
    }
}