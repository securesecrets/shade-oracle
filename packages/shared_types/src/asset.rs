use crate::scrt::{Api, CanonicalAddr, HumanAddr, StdError, Uint128, ContractLink, Extern, Storage, Querier, Env, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Contract {
    pub address: String,
    pub code_hash: String,
}

impl Contract {
    pub fn as_canonical(&self, api: &impl Api) -> Result<CanonicalContract, StdError> {
        Ok(CanonicalContract {
            address: api.canonical_address(&HumanAddr::from(self.address.clone()))?,
            code_hash: self.code_hash.clone(),
        })
    }

    pub fn new(link: &ContractLink<HumanAddr>) -> Self {
        Contract {
            address: link.address.to_string(),
            code_hash: link.code_hash.clone(),
        }
    }

    pub fn as_link(&self) -> ContractLink<HumanAddr> {
        ContractLink {
            address: HumanAddr(self.address.clone()),
            code_hash: self.code_hash.clone(),
        }
    }
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
            address: api.human_address(&self.address)?.to_string(),
            code_hash: self.code_hash.clone(),
        })
    }

    pub fn is_sender<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &mut Extern<S, A, Q>,
        env: &Env,
    ) -> StdResult<()> {
        if deps.api.canonical_address(&env.message.sender)? != self.address {
            return Err(StdError::Unauthorized { backtrace: None });
        } else {
            return Ok(());
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BorrowerInfo {
    pub address: String,
    pub collateral_amount: Uint128,
    pub loan_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TimeConstraints {
    pub block_time: u128,
    pub valid_timeframe: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ResponseStatus {
    Success,
    Failure,
}
