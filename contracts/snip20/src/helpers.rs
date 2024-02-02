use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, QuerierWrapper, StdResult, StdError};
use shade_toolkit::{Contract, Query};
use crate::msg::{QueryAnswer, QueryMsg};

#[cw_serde]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_supply: Option<Uint128>,
}

/// Returns a StdResult<TokenInfo> from performing TokenInfo query
pub fn query_token_info(querier: &QuerierWrapper, contract: &Contract) -> StdResult<TokenInfo> {
    let answer: QueryAnswer = QueryMsg::TokenInfo {}.query(querier, contract)?;
    match answer {
        QueryAnswer::TokenInfo {
            name,
            symbol,
            decimals,
            total_supply,
        } => Ok(TokenInfo {
            name,
            symbol,
            decimals,
            total_supply,
        }),
        _ => Err(StdError::generic_err("Wrong answer")), //TODO: better error
    }
}

pub fn query_token_balance(
    contract: &Contract,
    querier: &QuerierWrapper,
    address: impl Into<String>,
    key: impl Into<String>,
) -> StdResult<Uint128> {
    let answer: QueryAnswer = QueryMsg::Balance {
        address: address.into(),
        key: key.into(),
    }
    .query(querier, contract)?;
    match answer {
        QueryAnswer::Balance { amount } => Ok(amount),
        QueryAnswer::ViewingKeyError { msg } => Err(StdError::generic_err(msg)),
        _ => Err(StdError::generic_err(
            "Invalid response to query token balance.",
        )),
    }
}