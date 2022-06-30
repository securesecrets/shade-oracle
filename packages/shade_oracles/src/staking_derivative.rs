use crate::common::Contract;
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod shade {

    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct InitMsg {
        pub supported_key: String,
        pub underlying_symbol: String,
        pub staking_derivative: Contract,
        pub router: Contract,
    }

    // We define a custom struct for each query response
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct Config {
        pub supported_key: String,
        pub underlying_symbol: String,
        pub router: Contract,
        pub staking_derivative: Contract,
        pub enabled: bool,
    }

    pub mod querier {
        use cosmwasm_std::{to_binary, Querier, QueryRequest, StdResult, WasmQuery};

        use super::*;

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
        #[serde(rename_all = "snake_case")]
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

        /// validators and their weights
        #[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
        pub struct WeightedValidator {
            /// the validator's address
            pub validator: HumanAddr,
            /// the validator's weight in whole percents
            pub weight: u8,
        }

        #[derive(Serialize, Deserialize, JsonSchema, Debug)]
        #[serde(rename_all = "snake_case")]
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
        pub fn query_derivative_price(
            contract: &Contract,
            querier: &impl Querier,
        ) -> StdResult<Uint128> {
            let resp: StakingDerivativeQueryAnswer =
                querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: contract.address.clone(),
                    callback_code_hash: contract.code_hash.clone(),
                    msg: to_binary(&StakingDerivativeQueryMsg::StakingInfo { time: 0 })?,
                }))?;

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
