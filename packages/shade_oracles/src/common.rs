use std::{hash::{Hash}};

use crate::{
    band::ReferenceData,
};
use cosmwasm_std::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize}; 

pub const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetPrice { symbol: String },
    GetPrices { symbols: Vec<String> },
}

/// Every HandleMsg for each specific oracle type should include this
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    SetStatus { enabled: bool },
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct HandleStatusAnswer {
    pub status: ResponseStatus,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OraclePrice {
    pub symbol: String,
    pub price: ReferenceData,
}

impl OraclePrice {
    pub fn new(symbol: String, reference_data: ReferenceData) -> Self {
        OraclePrice { symbol, price: reference_data }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Contract {
    pub address: HumanAddr,
    pub code_hash: String,
}

impl Contract {
    pub fn new(address: String, code_hash: String) -> Self {
        Contract { address: HumanAddr(address), code_hash }
    }

    pub fn as_canonical(&self, api: &impl Api) -> Result<CanonicalContract, StdError> {
        Ok(CanonicalContract {
            address: api.canonical_address(&self.address.clone())?,
            code_hash: self.code_hash.clone(),
        })
    }
}

pub fn throw_unsupported_symbol_error(symbol: String) -> StdError {
    StdError::generic_err(format!("{} is not supported as a symbol.", symbol))
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CanonicalContract {
    pub address: CanonicalAddr,
    pub code_hash: String,
}

impl CanonicalContract {
    pub fn as_human(&self, api: &impl Api) -> Result<Contract, StdError> {
        Ok(Contract {
            address: api.human_address(&self.address)?,
            code_hash: self.code_hash.clone(),
        })
    }

    pub fn is_sender<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &mut Extern<S, A, Q>,
        env: &Env,
    ) -> StdResult<()> {
        if deps.api.canonical_address(&env.message.sender)? != self.address {
            Err(StdError::Unauthorized { backtrace: None })
        } else {
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[derive(Serialize, Deserialize)]
pub struct CommonOracleConfig {
    pub owner: HumanAddr,
    pub enabled: bool,
}

impl CommonOracleConfig {
    pub fn is_owner(&self, env: &Env) -> StdResult<&Self> {
        if env.message.sender != self.owner {
            Err(StdError::unauthorized())
        } else {
            Ok(self)
        }
    }

    pub fn is_enabled(&self) -> StdResult<&Self> {
        if self.enabled {
            Err(StdError::generic_err("This oracle has been disabled."))
        } else {
            Ok(self)
        }
    }
}

pub mod querier {
    use super::*;
    use fadroma::snip20_impl::msg as snip20;
    use secret_toolkit::snip20::TokenInfoResponse;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct Balance {
        pub amount: Uint128,
    }

    pub fn query_price(
        contract: &Contract,
        querier: &impl Querier,
        symbol: String,
    ) -> StdResult<OraclePrice> {
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.address.clone(),
            callback_code_hash: contract.code_hash.clone(),
            msg: to_binary(&QueryMsg::GetPrice { symbol })?,
        }))
    }

    pub fn query_prices(
        contract: &Contract,
        querier: &impl Querier,
        symbols: Vec<String>,
    ) -> StdResult<Vec<OraclePrice>> {
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.address.clone(),
            callback_code_hash: contract.code_hash.clone(),
            msg: to_binary(&QueryMsg::GetPrices { symbols })?,
        }))
    }

    pub fn query_token_info(
        contract: &Contract,
        querier: &impl Querier,
    ) -> StdResult<TokenInfoResponse> {
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.address.clone(),
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
            contract_addr: contract.address.clone(),
            callback_code_hash: contract.code_hash.clone(),
            msg: to_binary(&snip20::QueryMsg::Balance { address, key })?,
        }))?;
        match answer {
            snip20::QueryAnswer::Balance { amount } => Ok(Balance { amount }),
            snip20::QueryAnswer::ViewingKeyError { msg } => Err(StdError::generic_err(msg)),
            _ => Err(StdError::generic_err(
                "Invalid response to query token balance.",
            )),
        }
    }
}
