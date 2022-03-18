use super::EnsembleContract;
use crate::{ensemble_new, ensemblify};
use ::liquidation::contract as liq_contract;
use shade_lend::liquidation::{
    self, BidPoolResponse, BidResponse, ConfigResponse, HandleMsg::*, InitMsg, QueryMsg::*,
    Snip20HookMsg, TotalBidResponse,
};
use shared_types::{
    asset::Contract,
    composable_snip20::msg as snip20,
    ensemble::ContractEnsemble,
    scrt::{to_binary, ContractInstantiationInfo, HumanAddr, StdResult, Uint128},
    secret_toolkit::permit::Permit,
};
use std::{cell::RefCell, rc::Rc};

ensemblify!(
    LiqQueueHarness,
    liq_contract::init,
    liq_contract::handle,
    liq_contract::query,
    LiqQueue
);

impl LiqQueue {
    ensemble_new!(LiqQueue, InitMsg);

    pub fn update_config(&self, msg: &liquidation::HandleMsg, account_key: &str) -> StdResult<()> {
        self.handle(msg, Some(account_key))
    }

    pub fn place_bid(
        &self,
        account_key: &str,
        stablecoin: &impl EnsembleContract,
        amount: Uint128,
        premium: u8,
    ) -> StdResult<()> {
        let place_bid_msg = liquidation::Snip20HookMsg::PlaceBid {
            premium_slot: premium,
        };
        let snip20_msg = snip20::HandleMsg::Send {
            recipient: HumanAddr::from(self.info.address.clone()),
            msg: Some(to_binary(&place_bid_msg).unwrap()),
            memo: None,
            padding: None,
            amount: amount,
            recipient_code_hash: None,
        };
        stablecoin.handle(&snip20_msg, Some(account_key))
    }

    pub fn remove_bid(&self, account_key: &str, amount: Option<Uint128>, id: u64) -> StdResult<()> {
        let msg = liquidation::HandleMsg::RemoveBid {
            id: id,
            amount: amount,
        };
        self.handle(&msg, Some(account_key))
    }

    pub fn clean_bids(&self, account_key: &str, amount: Option<Uint128>) -> StdResult<()> {
        let msg = liquidation::HandleMsg::CleanBids { amount: amount };
        self.handle(&msg, Some(account_key))
    }

    pub fn activate_bid(&self, account_key: &str) -> StdResult<()> {
        let msg = liquidation::HandleMsg::ActivateBid {};
        self.handle(&msg, Some(account_key))
    }

    pub fn claim_collateral(&self, account_key: &str) -> StdResult<()> {
        let msg = liquidation::HandleMsg::ClaimCollateral {};
        self.handle(&msg, Some(account_key))
    }

    pub fn get_config(&self) -> ConfigResponse {
        let config: ConfigResponse = self.query(&GetConfig {}).unwrap();
        config
    }

    pub fn get_bid(&self, permit: Permit) -> Vec<liquidation::BidResponse> {
        let result: Vec<BidResponse> = self
            .query(&liquidation::QueryMsg::GetBid { permit: permit })
            .unwrap();
        result
    }

    pub fn get_bidpool(&self, premium_slot: u8) -> liquidation::BidPoolResponse {
        let result: BidPoolResponse = self
            .query(&liquidation::QueryMsg::GetBidPool {
                premium_slot: premium_slot,
            })
            .unwrap();
        result
    }

    pub fn get_total_bids(&self) -> Uint128 {
        let result: StdResult<TotalBidResponse> =
            self.query(&liquidation::QueryMsg::GetTotalBids {});
        match result {
            Ok(result) => result.total_bids,
            Err(msg) => {
                println!("{}", msg);
                let result = TotalBidResponse {
                    total_bids: Uint128(0),
                };
                result.total_bids
            }
        }
    }

    pub fn get_claimable_collateral(&self, permit: Permit) -> Vec<(Uint128, Uint128)> {
        let total_claimable: Vec<(Uint128, Uint128)> = self
            .query(&liquidation::QueryMsg::GetClaimableCollateral { permit: permit })
            .unwrap();
        total_claimable
    }

    pub fn get_remaining_bid(&self, permit: Permit) -> Vec<(Uint128, Uint128)> {
        let remaining_bid: Vec<(Uint128, Uint128)> = self
            .query(&liquidation::QueryMsg::GetRemainingBid { permit: permit })
            .unwrap();
        remaining_bid
    }

    pub fn print_liquidation_config(&self) -> StdResult<()> {
        let configResponse: ConfigResponse = self.get_config();
        println!(
            "CONFIG || {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
            configResponse.owner,
            configResponse.overseer,
            configResponse.vault,
            configResponse.treasury,
            configResponse.collateral_token,
            configResponse.stablecoin
        );
        println!(
            "CONFIG || {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
            configResponse.max_slots,
            configResponse.bid_fee,
            configResponse.liquidation_fee,
            configResponse.minimum_for_activation,
            configResponse.activation_delay,
            configResponse.collateral_token_decimals,
            configResponse.stablecoin_decimals
        );
        Ok(())
    }

    pub fn print_bid(&self, permit: Permit) -> StdResult<()> {
        let input: Vec<BidResponse> = self.get_bid(permit);
        for (pos, bid) in input.iter().enumerate() {
            println!("BID || ID {:?}, Stables {:?}, Premium {:?}, Claim {:?}, E {:?}, Scale {:?}, S {:?}, P {:?}, Time {:?}",
            bid.bid_id, bid.stable_amount, bid.premium_slot, bid.claimable_collateral, bid.epoch, bid.scale, bid.sum, bid.product, bid.place_time);
        }
        Ok(())
    }

    pub fn print_bidpool(&self, premium_slot: u8) -> StdResult<()> {
        let bidpool: BidPoolResponse = self.get_bidpool(premium_slot);
        println!(
            "BID POOL || S {:?}, P {:?}, E {:?}, Scale {:?}, Bids {:?}, Premium {:?}",
            bidpool.sum,
            bidpool.product,
            bidpool.current_epoch,
            bidpool.current_scale,
            bidpool.total_bids,
            bidpool.premium_slot
        );
        Ok(())
    }

    pub fn print_total_bids(&self) -> StdResult<()> {
        let total_bids: Uint128 = self.get_total_bids();
        println!("Total Bids {:?}", total_bids);
        Ok(())
    }

    pub fn print_claimable_collateral(&self, permit: Permit) -> StdResult<()> {
        let result: Vec<(Uint128, Uint128)> = self.get_claimable_collateral(permit);
        for (_pos, bid) in result.iter().enumerate() {
            println!("BID || ID {:?}, Claimable {:?}", bid.0, bid.1);
        }
        Ok(())
    }

    pub fn print_remaining_bid(&self, permit: Permit) -> StdResult<()> {
        let result: Vec<(Uint128, Uint128)> = self.get_remaining_bid(permit);
        for (_pos, bid) in result.iter().enumerate() {
            println!("BID || ID {:?}, Remaining Stables {:?}", bid.0, bid.1);
        }
        Ok(())
    }
}
