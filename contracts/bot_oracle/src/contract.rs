use cosmwasm_std::{
    entry_point, Deps, Env, Response, StdResult,
};
use cosmwasm_std::{DepsMut, MessageInfo, QueryResponse};
use shade_oracles::interfaces::common::config::BotPermission;
use shade_oracles::{interfaces::bot::*};

const ACTION_PREFIX: &str = "bot-oracle_";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    BotRateOracle::instantiate(deps, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    BotRateOracle::execute(deps, env, info, msg, ACTION_PREFIX)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    BotRateOracle::query(deps, env, msg)
}
