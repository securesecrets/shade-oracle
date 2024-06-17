use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, QuerierWrapper, StdError, Uint128, Uint256};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use shade_oracles::better_secret_math::common::exp10;
use shade_oracles::core::{pad_query_result, validate_admin, AdminPermissions};
use shade_oracles::interfaces::common::{OraclePrice, OracleQuery, PriceResponse};
use shade_oracles::interfaces::providers::ReferenceData;
use shade_oracles::ssp::Item;
use shade_toolkit::{Contract, Query, BLOCK_SIZE};

use crate::msg::*;

// Storage
const STAKING_INFO: Item<StakingInfoResponse> = Item::new("staking-info-response");

#[cw_serde]
pub struct InstantiateMsg {
    pub price: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let stakingInfo = StakingInfoResponse {
        unbonding_time: Uint128::zero(),
        bonded_shd: Uint128::zero(),
        rewards: Uint128::zero(),
        total_derivative_token_supply: Uint128::zero(),
        price: msg.price,
        fee_info: FeeInfo {
            staking: Fee {
                rate: 0,
                decimal_places: 0,
            },
            unbonding: Fee {
                rate: 0,
                decimal_places: 0,
            },
            collector: info.sender,
        },
        status: ContractStatusLevel::NormalRun,
    };

    STAKING_INFO.save(deps.storage, &stakingInfo)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::StakingInfo {} => to_binary(&STAKING_INFO.load(deps.storage)?),
        },
        BLOCK_SIZE,
    )
}
