use std::hash::Hash;
use crate::{
    band::ReferenceData,
};
use cosmwasm_std::*;
use cosmwasm_math_compat::{Uint128, Uint256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize}; 
use secret_toolkit::utils::{HandleCallback, Query};

pub const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetPrice { key: String },
    GetPrices { keys: Vec<String> },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

/// Every HandleMsg for each specific oracle type should include this
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    SetStatus { enabled: bool },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
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
    pub key: String,
    pub data: ReferenceData,
}

impl OraclePrice {
    pub fn new(key: String, reference_data: ReferenceData) -> Self {
        OraclePrice { key, data: reference_data }
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


pub fn get_precision(factor: u8) -> Uint128 {
    Uint128::from(10u128.pow(factor.into()))
}

pub fn throw_unsupported_symbol_error(key: String) -> StdError {
    StdError::generic_err(format!("{} is not supported as a key.", key))
}

pub fn normalize_price(amount: Uint128, decimals: u8) -> Uint128 {
    (amount.u128() * 10u128.pow(18u32 - u32::try_from(decimals).unwrap())).into()
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

pub fn sqrt(value: Uint256) -> StdResult<Uint256> {
    let mut z = Uint256::zero();

    if value.gt(&Uint256::from(3u128)) {
        z = value;
        let mut x = value.checked_div(Uint256::from(2u128))?.checked_add(Uint256::from(1u128))?;

        while x.lt(&z) {
            z = x;
            x = value
                .checked_div(x)?
                .checked_add(x)?
                .checked_div(Uint256::from(2u128))?;
        }
    } else if !value.is_zero() {
        z = Uint256::from(1u128);
    }

    Ok(z)
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
            Ok(self)
        } else {
            Err(StdError::generic_err("This oracle has been disabled."))
        }
    }
}

pub mod querier {
    use super::*;
    use secret_toolkit::snip20::{QueryMsg as Snip20QueryMsg, Balance, AuthenticatedQueryResponse, TokenInfoResponse};

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

    pub fn query_token_info(
        contract: &Contract,
        querier: &impl Querier,
    ) -> StdResult<TokenInfoResponse> {
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.address.clone(),
            callback_code_hash: contract.code_hash.clone(),
            msg: to_binary(&Snip20QueryMsg::TokenInfo {})?,
        }))
    }

    pub fn query_token_balance(
        contract: &Contract,
        querier: &impl Querier,
        address: HumanAddr,
        key: String,
    ) -> StdResult<Balance> {
        let answer: AuthenticatedQueryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.address.clone(),
            callback_code_hash: contract.code_hash.clone(),
            msg: to_binary(&Snip20QueryMsg::Balance { address, key })?,
        }))?;
        match answer {
            AuthenticatedQueryResponse::Balance { amount } => Ok(Balance { amount }),
            AuthenticatedQueryResponse::ViewingKeyError { msg } => Err(StdError::generic_err(msg)),
            _ => Err(StdError::generic_err(
                "Invalid response to query token balance.",
            )),
        }
    }
}
