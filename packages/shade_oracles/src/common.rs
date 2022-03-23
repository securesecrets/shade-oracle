use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use mulberry_utils::{
    common::types::{Contract},
    scrt::*,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetPrice {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceResponse {
    pub rate: Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

pub fn query_price(contract: &Contract, querier: &impl Querier) -> StdResult<PriceResponse> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: HumanAddr(contract.address.clone()),
        callback_code_hash: contract.code_hash.clone(),
        msg: to_binary(&QueryMsg::GetPrice {})?,
    }))
}