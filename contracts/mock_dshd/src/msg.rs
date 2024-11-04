use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, Addr, QuerierWrapper, StdError, Storage, Uint128, Uint256};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use shade_oracles::better_secret_math::common::exp10;
use shade_oracles::core::{pad_query_result, validate_admin, AdminPermissions, ResponseStatus};
use shade_oracles::impl_msg_callbacks;
use shade_oracles::interfaces::common::{OraclePrice, OracleQuery, PriceResponse};
use shade_oracles::interfaces::providers::ReferenceData;
use shade_oracles::ssp::{Item, Map};
use shade_toolkit::{Contract, RawContract, BLOCK_SIZE};

impl_msg_callbacks!();

#[cw_serde]
pub struct InstantiateMsg {
    pub price: Uint128,
}

#[cw_serde]
pub enum QueryMsg {
    StakingInfo {},
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct Fee {
    pub rate: u32,
    pub decimal_places: u8,
}

#[cw_serde]
pub struct FeeInfo {
    pub staking: Fee,
    pub unbonding: Fee,
    pub collector: Addr,
}

#[cw_serde]
pub enum ContractStatusLevel {
    NormalRun,
    Panicked,
    StopAll,
}

#[cw_serde]
pub enum QueryResponse {
    StakingInfo {
        unbonding_time: Uint128,
        bonded_shd: Uint128,
        rewards: Uint128,
        total_derivative_token_supply: Uint128,
        price: Uint128,
        fee_info: FeeInfo,
        status: ContractStatusLevel,
    },
}

#[cw_serde]
pub struct StakingInfoResponse {
    pub unbonding_time: Uint128,
    pub bonded_shd: Uint128,
    pub rewards: Uint128,
    pub total_derivative_token_supply: Uint128,
    pub price: Uint128,
    pub fee_info: FeeInfo,
    pub status: ContractStatusLevel,
}
