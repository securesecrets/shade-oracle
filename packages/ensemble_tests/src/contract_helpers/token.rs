use super::EnsembleContract;
use crate::{constants::VIEW_KEY, ensemble_new, ensemblify};
use shared_types::{
    asset::Contract,
    composable_snip20::msg as snip20msg,
    ensemble::ContractEnsemble,
    scrt::{Binary, ContractInstantiationInfo, HumanAddr, StdResult, Uint128},
    secret_toolkit::snip20,
};
use std::{cell::RefCell, rc::Rc};

ensemblify!(
    Snip20Harness,
    ::snip20::init,
    ::snip20::handle,
    ::snip20::query,
    Snip20
);

impl Snip20 {
    ensemble_new!(Snip20, snip20msg::InitMsg);

    pub fn get_balance(&self, address: String, viewing_key: Option<String>) -> StdResult<Uint128> {
        let balance: snip20msg::QueryAnswer = self
            .query(&snip20::QueryMsg::Balance {
                address: HumanAddr(address),
                key: viewing_key.unwrap_or(VIEW_KEY.to_string()),
            })
            .unwrap();

        match balance {
            snip20msg::QueryAnswer::Balance { amount } => Ok(amount),
            _ => Ok(Uint128(0u128)),
        }
    }

    pub fn mint(&self, recipient: String, amount: u128, sender_key: Option<&str>) -> StdResult<()> {
        self.handle(
            &snip20::HandleMsg::Mint {
                recipient: HumanAddr(recipient.clone()),
                amount: Uint128(amount),
                memo: None,
                padding: None,
            },
            sender_key,
        )
    }

    pub fn send(
        &self,
        recipient: String,
        code_hash: Option<String>,
        amount: u128,
        hook: Option<Binary>,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        self.handle(
            &snip20::HandleMsg::Send {
                recipient: HumanAddr(recipient.clone()),
                recipient_code_hash: code_hash,
                amount: Uint128(amount),
                msg: hook,
                memo: None,
                padding: None,
            },
            sender_key,
        )
    }

    pub fn set_viewing_key(&self, viewing_key: &str, sender_key: &str) -> StdResult<()> {
        self.handle(
            &snip20::HandleMsg::SetViewingKey {
                key: viewing_key.to_string(),
                padding: None,
            },
            Some(sender_key),
        )
    }
}
