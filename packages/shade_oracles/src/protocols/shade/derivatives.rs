use super::*;

pub struct ShadeStakingDerivative;

impl ShadeStakingDerivative {
    /// Returns the price of 1 derivative token in underlying token (6 decimals)
    pub fn query_staking_info(
        querier: &QuerierWrapper,
        contract: &Contract,
    ) -> StdResult<StakingInfoResponse> {
        StakingDerivativeQueryMsg::StakingInfo { time: 0 }.query(querier, contract)
    }
}

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
    const BLOCK_SIZE: usize = BLOCK_SIZE;
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
pub struct StakingInfoResponse {
    /// validator addresses and their weights
    pub validators: Vec<WeightedValidator>,
    /// unbonding time
    pub unbonding_time: u32,
    /// minimum number of seconds between unbonding batches
    pub unbonding_batch_interval: u32,
    /// earliest time of next batch unbonding
    pub next_unbonding_batch_time: u64,
    /// amount of SCRT that will unbond in the next batch
    pub unbond_amount_of_next_batch: Uint128,
    /// true if a batch unbonding is in progress
    pub batch_unbond_in_progress: bool,
    /// amount of bonded SCRT
    pub bonded_scrt: Uint128,
    /// amount of SCRT reserved for mature unbondings
    pub reserved_scrt: Uint128,
    /// amount of available SCRT not reserved for mature unbondings
    pub available_scrt: Uint128,
    /// unclaimed staking rewards
    pub rewards: Uint128,
    /// total supply of derivative token
    pub total_derivative_token_supply: Uint128,
    /// price of derivative token in SCRT to 6 decimals
    pub price: Uint128,
}
