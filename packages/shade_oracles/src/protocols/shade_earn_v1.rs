use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Querier, QueryRequest, StdError, StdResult, Uint128, WasmQuery, QuerierWrapper};


use shade_protocol::utils::asset::{RawContract, Contract};
#[cw_serde]
pub enum QueryMsg {
    GetConfig { r#type: ConfigType },
    GetDepositForShares { amount: Uint128 },
    GetSharesForDeposit { amount: Uint128 },
    CheckReward { timestamp: Option<Uint128> },
    EstimateDeployedBalance,
    EstimateReinvestReward { timestamp: Option<Uint128> },
}

#[cw_serde]
pub enum ConfigType {
    General,
    Specific,
}

#[cw_serde]
pub struct DeveloperInfo {
    pub address: String,
    pub fee: Uint128,
}

#[cw_serde]
pub struct TreasuryInfo {
    pub address: String,
    pub fee: Uint128,
}

#[cw_serde]
pub enum QueryAnswer {
    DepositForShares { amount: Uint128 },
    SharesForDeposit { amount: Uint128 },
    Reward { amount: Uint128 },
    EstimatedDeployedBalance { amount: Uint128 },
    EstimatedReinvestReward { amount: Uint128 },
}

#[cw_serde]
pub struct GenericConfig {
    pub min_tokens_to_reinvest: Uint128,
    pub treasury_info: TreasuryInfo,
    pub dev_info: DeveloperInfo,
    pub reinvest_reward: Uint128,
    pub deposits_enabled: bool,
    pub owner: String,
    pub deposit_token: Contract,
    pub share_token: Contract,
}

pub fn query_deposit_for_shares(
    contract: &Contract,
    querier: &QuerierWrapper,
    amount: Uint128,
) -> StdResult<Uint128> {
    let result: QueryAnswer = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract.address.to_string(),
        code_hash: contract.code_hash.clone(),
        msg: to_binary(&QueryMsg::GetDepositForShares { amount })?,
    }))?;

    match result {
        QueryAnswer::DepositForShares { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            "An error occurred while trying to retrieve deposit for shares.",
        )),
    }
}

pub fn query_generic_config(
    contract: &Contract,
    querier: &QuerierWrapper,
) -> StdResult<GenericConfig> {
    let config_response: GenericConfig = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract.address.to_string(),
        code_hash: contract.code_hash.clone(),
        msg: to_binary(&QueryMsg::GetConfig {
            r#type: ConfigType::General,
        })?,
    }))?;
    Ok(config_response)
}
