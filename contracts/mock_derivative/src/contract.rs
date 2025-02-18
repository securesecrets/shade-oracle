use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, QuerierWrapper, StdError, Uint128, Uint256};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use shade_oracles::core::pad_query_result;
use shade_oracles::ssp::Item;
use shade_toolkit::BLOCK_SIZE;

use crate::msg::*;

// Storage
const STAKING_INFO: Item<StakingInfoResponse> = Item::new("staking-info-response");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let staking_info = StakingInfoResponse {
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

    STAKING_INFO.save(deps.storage, &staking_info)?;

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
