use cosmwasm_std::{entry_point, Deps, Env, Response, StdResult};
use cosmwasm_std::{DepsMut, MessageInfo, QueryResponse};
use shade_oracles::interfaces::common::config::BotPermission;
use shade_oracles::interfaces::derivatives::bot::*;

const ACTION_PREFIX: &str = "quicksilver-staking-derivatives-oracle_";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    BotStakingDerivativesOracle::instantiate(deps, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    BotStakingDerivativesOracle::execute(
        deps,
        env,
        info,
        msg,
        BotPermission::UpdateQuicksilverRates,
        ACTION_PREFIX,
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    BotStakingDerivativesOracle::query(deps, env, msg)
}
