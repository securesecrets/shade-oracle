pub use shade_protocol::{
    utils::generic_response::ResponseStatus
};
use std::hash::Hash;
use crate::{
    band::ReferenceData,
};
use cosmwasm_std::*;
use cosmwasm_math_compat::{Uint128, Uint256};
use fadroma::prelude::ContractLink;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize}; 
use secret_toolkit::utils::{Query};

pub const BLOCK_SIZE: usize = 256;

/// Default Query API for all oracles.
/// 
/// Every oracle must support these 3 methods in addition to any specific ones it wants to support.
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig { enabled: bool },
}

/// Default HandleAnswer for oracles if only HandleMsg implemented is UpdateConfig.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateConfig { status: ResponseStatus },
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

    pub fn new_link(link: ContractLink<HumanAddr>) -> Self {
        Contract { address: link.address, code_hash: link.code_hash }
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

pub fn is_disabled(enabled: bool) -> StdResult<()> {
    if !enabled {
        Err(StdError::generic_err("Deprecated oracle."))
    } else {
        Ok(())
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

pub mod querier {
    use std::collections::HashMap;

    use crate::router::{
        QueryMsg as RouterQueryMsg, OracleResponse,
        AdminAuthResponse
    };
    use super::*;
    use secret_toolkit::snip20::{QueryMsg as Snip20QueryMsg, Balance, AuthenticatedQueryResponse, TokenInfoResponse};
    use shade_admin::admin::{
        ValidateAdminPermissionResponse, QueryMsg as AdminQueryMsg
    };

    pub fn query_oracle_price(
        oracle: &Contract,
        querier: &impl Querier,
        key: String,
    ) -> StdResult<OraclePrice> {
        QueryMsg::GetPrice { key }.query(querier, oracle.code_hash.clone(), oracle.address.clone())
    }

    /// Gets the oracle for the key from the router & calls GetPrice on it.
    /// 
    /// Has a query depth of 1.
    pub fn query_price(
        router: &Contract,
        querier: &impl Querier,
        key: String,
    ) -> StdResult<OraclePrice> {
        let oracle_resp: OracleResponse = RouterQueryMsg::GetOracle { key: key.clone() }.query(querier, router.code_hash.clone(), router.address.clone())?;
        query_oracle_price(&oracle_resp.oracle, querier, key)
    }

    pub fn query_oracle_prices(
        oracle: &Contract,
        querier: &impl Querier,
        keys: Vec<String>,
    ) -> StdResult<Vec<OraclePrice>> {
        QueryMsg::GetPrices { keys }.query(querier, oracle.code_hash.clone(), oracle.address.clone())
    }
    
    /// Groups the keys by their respective oracles and sends bulk GetPrices queries to each of those oracles.
    /// 
    /// Done to reduce impact on query depth.
    pub fn query_prices(
        router: &Contract,
        querier: &impl Querier,
        keys: Vec<String>,
    ) -> StdResult<Vec<OraclePrice>> {
        let oracle_resps: Vec<OracleResponse> = RouterQueryMsg::GetOracles { keys }.query(querier, router.code_hash.clone(), router.address.clone())?;
        let mut map: HashMap<Contract, Vec<String>> = HashMap::new();
        let mut prices: Vec<OraclePrice> = vec![];

        for resp in oracle_resps {
            // Get the current vector of symbols at that oracle and add the current key to it
            map.entry(resp.oracle).or_insert(vec![]).push(resp.key);
        }
    
        for (oracle, keys) in map {
            if keys.len() == 1 {
                let queried_price = query_oracle_price(&oracle, querier, keys[0].clone())?;
                prices.push(queried_price);
            } else {
                let mut queried_prices = query_oracle_prices(&oracle, querier, keys)?;
                prices.append(&mut queried_prices);
            }
        }
        Ok(prices)
    }

    /// Gets the admin auth contract from the router and uses it to check if the user is an admin for the router.
    pub fn verify_admin(
        contract: &Contract,
        querier: &impl Querier,
        user: HumanAddr,
    ) -> StdResult<()> {
        let get_admin_auth_req: AdminAuthResponse = RouterQueryMsg::GetAdminAuth {  }.query(querier, contract.code_hash.clone(), contract.address.clone())?; 
        let admin_auth = get_admin_auth_req.admin_auth;
        let is_admin_req: ValidateAdminPermissionResponse = AdminQueryMsg::ValidateAdminPermission { contract_address: contract.address.to_string(), admin_address: user.to_string() }.query(querier, admin_auth.code_hash, admin_auth.address)?;
        match is_admin_req.error_msg {
            Some(err) => Err(StdError::generic_err(err)),
            None => Ok(()),
        }
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

