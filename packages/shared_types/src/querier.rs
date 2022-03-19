use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::{
    asset::Contract,
    composable_snip20::msg as snip20,
    scrt::{Uint128, to_binary, HumanAddr, Querier, QueryRequest, StdError, StdResult, WasmQuery},
    secret_toolkit::snip20::{TokenInfoResponse},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Balance {
    pub amount: Uint128,
}

pub fn query_token_info(
    contract: &Contract,
    querier: &impl Querier,
) -> StdResult<TokenInfoResponse> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: HumanAddr(contract.address.clone()),
        callback_code_hash: contract.code_hash.clone(),
        msg: to_binary(&snip20::QueryMsg::TokenInfo {})?,
    }))
}

pub fn query_token_balance(
    contract: &Contract,
    querier: &impl Querier,
    address: HumanAddr,
    key: String,
) -> StdResult<Balance> {
    let answer: snip20::QueryAnswer = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: HumanAddr(contract.address.clone()),
        callback_code_hash: contract.code_hash.clone(),
        msg: to_binary(&snip20::QueryMsg::Balance { address, key })?,
    }))?;
    match answer {
        snip20::QueryAnswer::Balance { amount } => Ok(Balance { amount: amount }),
        snip20::QueryAnswer::ViewingKeyError { msg } => Err(StdError::generic_err(msg)),
        _ => Err(StdError::generic_err(
            "Invalid response to query token balance.",
        )),
    }
}
