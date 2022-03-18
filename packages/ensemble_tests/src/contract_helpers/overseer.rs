use ::overseer::contract as overseer_contract;
use shade_lend::overseer;
use shared_types::{
    asset::Contract,
    composable_snip20::msg as snip20,
    ensemble::ContractEnsemble,
    scrt::{to_binary, ContractInstantiationInfo, HumanAddr, StdResult, Uint128},
};
use std::{cell::RefCell, rc::Rc};

use crate::{ensemble_new, ensemblify};

use super::EnsembleContract;

ensemblify!(
    LendOverseerHarness,
    overseer_contract::init,
    overseer_contract::handle,
    overseer_contract::query,
    LendOverseer
);

impl LendOverseer {
    ensemble_new!(LendOverseer, overseer::InitMsg);

    pub fn get_config(&self) -> overseer::ConfigResponse {
        self.query(&overseer::QueryMsg::GetConfig {}).unwrap()
    }

    pub fn get_asset_balance(&self, asset: &impl EnsembleContract) -> Uint128 {
        let msg = overseer::QueryMsg::GetAssetBalance {
            asset: asset.as_contract(),
        };

        let balance: overseer::AssetBalanceResponse = self.query(&msg).unwrap();

        balance.amount
    }

    pub fn whitelist_vault(
        &self,
        vault: &impl EnsembleContract,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        let msg = overseer::HandleMsg::WhitelistVault {
            vault: vault.as_contract(),
        };
        self.handle(&msg, sender_key)
    }

    pub fn increase_silk_in_vault(
        &self,
        vault: &impl EnsembleContract,
        amount: u128,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        let msg = overseer::HandleMsg::IncreaseSilkInVault {
            vault: vault.as_contract().address,
            amount: Uint128(amount),
        };
        self.handle(&msg, sender_key)
    }

    pub fn deposit_vault(
        &self,
        vault: &impl EnsembleContract,
        token: &impl EnsembleContract,
        amount: Uint128,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        let deposit_msg = overseer::Snip20HookMsg::DepositIntoVault {
            vault: vault.as_contract().address,
        };
        let snip20_msg = snip20::HandleMsg::Send {
            recipient: HumanAddr::from(self.info.address.clone()),
            msg: Some(to_binary(&deposit_msg).unwrap()),
            memo: None,
            padding: None,
            amount: amount,
            recipient_code_hash: None,
        };
        token.handle(&snip20_msg, sender_key)
    }

    pub fn withdraw_vault(
        &self,
        vault: &impl EnsembleContract,
        amount: Uint128,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        let msg = overseer::HandleMsg::WithdrawFromVault {
            vault: vault.as_contract().address,
            amount: amount,
        };
        self.handle(&msg, sender_key)
    }

    pub fn repay_vault(
        &self,
        vault: &impl EnsembleContract,
        token: &impl EnsembleContract,
        amount: Uint128,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        let repay_msg = overseer::Snip20HookMsg::Repay {
            vault: vault.as_contract().address,
        };
        let snip20_msg = snip20::HandleMsg::Send {
            recipient: HumanAddr::from(self.info.address.clone()),
            msg: Some(to_binary(&repay_msg).unwrap()),
            memo: None,
            padding: None,
            amount: amount,
            recipient_code_hash: None,
        };
        token.handle(&snip20_msg, sender_key)
    }

    pub fn borrow_vault(
        &self,
        vault: &impl EnsembleContract,
        amount: Uint128,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        let msg = overseer::HandleMsg::BorrowFromVault {
            vault: vault.as_contract().address,
            amount: amount,
        };
        self.handle(&msg, sender_key)
    }
}
