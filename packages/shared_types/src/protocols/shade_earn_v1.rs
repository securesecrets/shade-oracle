use fadroma::{Querier, Uint128, StdResult, QueryRequest, WasmQuery, HumanAddr, to_binary, StdError};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::asset::Contract;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum QueryMsg {
    GetConfig { r#type: ConfigType },
    GetDepositForShares { amount: Uint128 },
    GetSharesForDeposit { amount: Uint128 },
    CheckReward { timestamp: Option<Uint128> },
    EstimateDeployedBalance,
    EstimateReinvestReward { timestamp: Option<Uint128> },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum ConfigType {
    General,
    Specific,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct DeveloperInfo {
    pub address: String,
    pub fee: Uint128,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct TreasuryInfo {
    pub address: String,
    pub fee: Uint128,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum QueryAnswer {
    DepositForShares { amount: Uint128 },
    SharesForDeposit { amount: Uint128 },
    Reward { amount: Uint128 },
    EstimatedDeployedBalance { amount: Uint128 },
    EstimatedReinvestReward { amount: Uint128 },
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
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
    querier: &impl Querier,
    amount: Uint128,
) -> StdResult<Uint128> {
    let result: QueryAnswer = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: HumanAddr(contract.address.clone()),
        callback_code_hash: contract.code_hash.clone(),
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
    querier: &impl Querier,
) -> StdResult<GenericConfig> {
    let config_response: GenericConfig = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: HumanAddr::from(contract.address.clone()),
        callback_code_hash: contract.code_hash.clone(),
        msg: to_binary(&QueryMsg::GetConfig {
            r#type: ConfigType::General,
        })?,
    }))?;
    Ok(config_response)
}
