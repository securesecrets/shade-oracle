use crate::scrt::*;
use crate::scrt::secret_toolkit::snip20;
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

#[derive(Clone, Debug)]
pub struct ISnip20 {
    pub link:   ContractLink<HumanAddr>,
    padding:    Option<String>,
    memo:       Option<String>,
    block_size: usize
}

impl ISnip20 {

    pub fn attach (link: ContractLink<HumanAddr>) -> Self {
        Self {
            link,
            padding:    None,
            memo: None,
            block_size: BLOCK_SIZE
        }
    }

    pub fn attach_to (address: &HumanAddr, code_hash: &String) -> Self {
        Self {
            link: ContractLink { address: address.clone(), code_hash: code_hash.clone() },
            padding:    None,
            memo:       None,
            block_size: BLOCK_SIZE
        }
    }

    pub fn memo(mut self, memo: String) -> Self {
        self.memo = Some(memo);

        self
    }

    pub fn mint (
        &self, recipient: &HumanAddr, amount: Uint128
    ) -> StdResult<CosmosMsg> {
        snip20::mint_msg(
            recipient.clone(), amount,
            self.padding.clone(), self.block_size,
            self.link.code_hash.clone(), self.link.address.clone()
        )
    }

    pub fn burn (
        &self, amount: Uint128
    ) -> StdResult<CosmosMsg> {
        snip20::burn_msg(
            amount,
            self.padding.clone(), self.block_size,
            self.link.code_hash.clone(), self.link.address.clone()
        )
    }

    pub fn set_minters (
        &self, minters: &Vec<HumanAddr>
    ) -> StdResult<CosmosMsg> {
        snip20::set_minters_msg(
            minters.clone(),
            self.padding.clone(), self.block_size,
            self.link.code_hash.clone(), self.link.address.clone()
        )
    }

    pub fn send (
        &self, recipient: &HumanAddr, amount: Uint128, msg: Option<Binary>
    ) -> StdResult<CosmosMsg> {
        snip20::send_msg(
            recipient.clone(), amount, msg,
            self.padding.clone(), self.block_size,
            self.link.code_hash.clone(), self.link.address.clone()
        )
    }

    pub fn send_from (
        &self, owner: &HumanAddr, recipient: &HumanAddr,
        amount: Uint128, msg: Option<Binary>
    ) -> StdResult<CosmosMsg> {
        snip20::send_from_msg(
            owner.clone(), recipient.clone(), amount, msg,
             self.padding.clone(), self.block_size,
            self.link.code_hash.clone(), self.link.address.clone()
        )
    }

    pub fn register_receive (&self, hash: String) -> StdResult<CosmosMsg> {
        snip20::register_receive_msg(
            hash,
            self.padding.clone(), self.block_size,
            self.link.code_hash.clone(), self.link.address.clone()
        )
    }

    pub fn transfer (
        &self, recipient: &HumanAddr, amount: Uint128
    ) -> StdResult<CosmosMsg> {
        snip20::transfer_msg(
            recipient.clone(), amount,
            self.padding.clone(), self.block_size,
            self.link.code_hash.clone(), self.link.address.clone()
        )
    }

    pub fn transfer_from (
        &self, owner: &HumanAddr, recipient: &HumanAddr, amount: Uint128
    ) -> StdResult<CosmosMsg> {
        snip20::transfer_from_msg(
            owner.clone(), recipient.clone(), amount,
             self.padding.clone(), self.block_size,
            self.link.code_hash.clone(), self.link.address.clone()
        )
    }

    pub fn set_viewing_key (
        &self, vk: &str
    ) -> StdResult<CosmosMsg> {
        snip20::set_viewing_key_msg(
            vk.into(),
            None, BLOCK_SIZE,
            self.link.code_hash.clone(), self.link.address.clone()
        )
    }

    pub fn increase_allowance (
        &self, recipient: &HumanAddr, amount: Uint128, duration: Option<u64>
    ) -> StdResult<CosmosMsg> {
        snip20::increase_allowance_msg(
            recipient.clone(), amount, duration,
            None, BLOCK_SIZE,
            self.link.code_hash.clone(), self.link.address.clone()
        )
    }

    pub fn query_balance (
        &self, querier: &impl Querier, address: &HumanAddr, vk: &str
    ) -> StdResult<Uint128> {
        Ok(snip20::balance_query(
            querier, address.clone(), vk.into(),
            self.block_size, self.link.code_hash.clone(), self.link.address.clone()
        )?.amount)
    }

    pub fn query_token_info (
        &self, querier: &impl Querier
    ) -> StdResult<snip20::TokenInfo> {
        snip20::token_info_query(
            querier,
            self.block_size, self.link.code_hash.clone(), self.link.address.clone()
        )
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
