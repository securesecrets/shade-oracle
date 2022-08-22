use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cosmwasm_std::Uint128;
use shade_protocol::utils::asset::{Contract, RawContract};
#[cfg(feature = "core")]
use crate::ssp::{Item, ItemStorage};
use shade_protocol::utils::InstantiateCallback;

pub mod shade {

    use crate::{common::InstantiateCommonConfig, BLOCK_SIZE};

    use super::*;

    #[cw_serde]
    /// Config needs symbol of underlying, one supported key,
    /// and a dependency named "staking-derivative-token".
    pub struct InstantiateMsg {
        pub config: InstantiateCommonConfig,
        pub staking_derivative_token: RawContract,
        pub underlying_symbol: String,
    }

    impl InstantiateCallback for InstantiateMsg {
        const BLOCK_SIZE: usize = BLOCK_SIZE;
    }

    #[cw_serde]
    /// token_decimals - # of decimals used by staking derivative token
    ///
    /// staking_derivative_token - token this oracle is for
    ///
    /// underlying_symbol - the symbol of the underlying asset for which the token is a derivative (used to query its price via router or band)
    pub struct StakingDerivativeConfig {
        pub token_decimals: u8,
        pub staking_derivative_token: Contract,
        pub underlying_symbol: String,
    }

    #[cfg(feature = "shd-staking-derivative")]
    impl ItemStorage for StakingDerivativeConfig {
        const ITEM: Item<'static, Self> = Item::new("staking-derivative-config");
    }

    pub mod querier {
        use cosmwasm_std::{Deps, StdResult};
        use shade_protocol::utils::Query;

        use super::*;

        #[cw_serde]
        pub enum StakingDerivativeQueryMsg {
            // staking
            /// display the validator addresses, amount of bonded SCRT, amount of available SCRT not
            /// reserved for mature unbondings, amount of pending staking rewards not yet claimed,
            /// the derivative token supply, and the price of the derivative token in SCRT to 6 decimals
            StakingInfo {
                /// time in seconds since 01/01/1970.
                time: u64,
            },
        }

        impl Query for StakingDerivativeQueryMsg {
            const BLOCK_SIZE: usize = 256;
        }

        /// validators and their weights
        #[cw_serde]
        pub struct WeightedValidator {
            /// the validator's address
            pub validator: Addr,
            /// the validator's weight in whole percents
            pub weight: u8,
        }

        #[cw_serde]
        pub enum StakingDerivativeQueryAnswer {
            // Staking
            /// displays staking info
            StakingInfo {
                /// validator addresses and their weights
                validators: Vec<WeightedValidator>,
                /// unbonding time
                unbonding_time: u32,
                /// minimum number of seconds between unbonding batches
                unbonding_batch_interval: u32,
                /// earliest time of next batch unbonding
                next_unbonding_batch_time: u64,
                /// amount of SCRT that will unbond in the next batch
                unbond_amount_of_next_batch: Uint128,
                /// true if a batch unbonding is in progress
                batch_unbond_in_progress: bool,
                /// amount of bonded SCRT
                bonded_scrt: Uint128,
                /// amount of SCRT reserved for mature unbondings
                reserved_scrt: Uint128,
                /// amount of available SCRT not reserved for mature unbondings
                available_scrt: Uint128,
                /// unclaimed staking rewards
                rewards: Uint128,
                /// total supply of derivative token
                total_derivative_token_supply: Uint128,
                /// price of derivative token in SCRT to 6 decimals
                price: Uint128,
            },
        }
        /// Returns the price of 1 derivative token in underlying token (6 decimals)
        pub fn query_derivative_price(deps: Deps, contract: &Contract) -> StdResult<Uint128> {
            let resp: StakingDerivativeQueryAnswer =
                StakingDerivativeQueryMsg::StakingInfo { time: 0 }
                    .query(&deps.querier, contract)?;

            match resp {
                StakingDerivativeQueryAnswer::StakingInfo {
                    validators: _,
                    unbonding_time: _,
                    unbonding_batch_interval: _,
                    next_unbonding_batch_time: _,
                    unbond_amount_of_next_batch: _,
                    batch_unbond_in_progress: _,
                    bonded_scrt: _,
                    reserved_scrt: _,
                    available_scrt: _,
                    rewards: _,
                    total_derivative_token_supply: _,
                    price,
                } => Ok(price),
            }
        }
    }
}
