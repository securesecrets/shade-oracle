use super::EnsembleContract;
use crate::{constants::VIEW_KEY, ensemble_new, ensemblify};
use shade_lend::vault::{self, VaultPositionInfoResponse};
use shared_types::{
    asset::Contract,
    composable_snip20::msg as snip20msg,
    ensemble::ContractEnsemble,
    scrt::{Binary, ContractInstantiationInfo, HumanAddr, StdResult, Uint128},
    secret_toolkit::{permit::Permit, snip20},
};
use std::{cell::RefCell, rc::Rc};

ensemblify!(
    VaultHarness,
    ::vault::contract::init,
    ::vault::contract::handle,
    ::vault::contract::query,
    Vault
);

impl Vault {
    ensemble_new!(Vault, vault::InitMsg);

    pub fn update_config(&self, msg: &vault::HandleMsg, account_key: &str) -> StdResult<()> {
        self.handle(msg, Some(account_key))
    }

    pub fn execute_liquidation(
        &self,
        account_key: &str,
        liquidator: String,
        borrower: u64,
    ) -> StdResult<()> {
        let msg = vault::HandleMsg::ExecuteLiquidation {
            liquidator: liquidator,
            borrower: borrower,
        };
        self.handle(&msg, Some(account_key))
    }

    pub fn get_config(&self) -> vault::Config {
        let config: vault::Config = self.query(&vault::QueryMsg::GetConfig {}).unwrap();
        config
    }

    pub fn get_parameters(&self) -> vault::Parameters {
        let parameters: vault::Parameters = self.query(&vault::QueryMsg::GetParameters {}).unwrap();
        parameters
    }

    pub fn get_vault_position(&self, permit: Permit) -> vault::VaultPositionInfoResponse {
        let vault_position = self.query(&vault::QueryMsg::GetVaultPositionInfo { permit });
        match vault_position {
            Ok(vault) => vault,
            Err(msg) => {
                println!("{}", msg);
                VaultPositionInfoResponse {
                    collateral_deposited: Uint128(0),
                    collateral_value: Uint128(0),
                    position_apy: Uint128(0),
                    loan_amount: Uint128(0),
                    loan_value: Uint128(0),
                    ltv: Uint128(0),
                }
            }
        }
    }

    pub fn get_accrued_fees(&self) -> vault::VaultAccruedFeesResponse {
        let vault_fees: vault::VaultAccruedFeesResponse =
            self.query(&vault::QueryMsg::GetAccruedFees {}).unwrap();
        vault_fees
    }

    pub fn get_liquidatable_positions(&self, size: Option<Uint128>) -> Vec<u64> {
        let liquidatable_positions: Vec<u64> = self
            .query(&vault::QueryMsg::GetLiquidatablePositions { size: size })
            .unwrap();
        liquidatable_positions
    }

    pub fn print_vault_position(&self, permit: Permit) -> StdResult<()> {
        let vault_position: vault::VaultPositionInfoResponse =
            self.get_vault_position(permit.clone());
        println!("User's loan: {}", vault_position.loan_amount);
        println!("User's loan value: {}", vault_position.loan_value);
        println!("User's LTV: {}", vault_position.ltv);
        println!("User's collateral: {}", vault_position.collateral_deposited);
        println!(
            "User's collateral value: {}",
            vault_position.collateral_value
        );
        Ok(())
    }
}
