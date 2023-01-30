use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, DepsMut, MessageInfo, Uint128};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Response, StdResult};
use shade_oracles::{
    core::{ExecuteCallback, InstantiateCallback},
    protocols::shade_staking_derivatives::{StakingDerivativeQueryMsg, StakingInfoResponse},
    ssp::Item,
};

#[cw_serde]
pub struct InstantiateMsg {}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

const STAKING_INFO: Item<StakingInfoResponse> = Item::new("pair_info");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[cw_serde]
pub enum ExecuteMsg {
    SetPrice(Uint128),
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SetPrice(price) => {
            STAKING_INFO.save(
                deps.storage,
                &StakingInfoResponse {
                    validators: vec![],
                    unbonding_time: 0u32,
                    unbonding_batch_interval: 0u32,
                    next_unbonding_batch_time: 0u64,
                    unbond_amount_of_next_batch: Uint128::zero(),
                    batch_unbond_in_progress: false,
                    bonded_scrt: Uint128::zero(),
                    reserved_scrt: Uint128::zero(),
                    available_scrt: Uint128::zero(),
                    rewards: Uint128::zero(),
                    total_derivative_token_supply: Uint128::zero(),
                    price,
                },
            )?;

            Ok(Response::default())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: StakingDerivativeQueryMsg) -> StdResult<Binary> {
    match msg {
        StakingDerivativeQueryMsg::StakingInfo { .. } => {
            to_binary(&STAKING_INFO.load(deps.storage)?)
        }
    }
}
