use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, DepsMut, MessageInfo, Uint128};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Response, StdResult};
use shade_oracles::{
    core::InstantiateCallback,
    protocols::shade_staking_derivatives::{StakingDerivativeQueryMsg, StakingInfoResponse},
    ssp::Item,
};
use snip20::helpers::TokenInfo;
use snip20::msg::QueryAnswer;

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub price: Uint128,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

const STAKING_INFO: Item<StakingInfoResponse> = Item::new("pair_info");
const TOKEN_INFO: Item<TokenInfo> = Item::new("query_token_info");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let query_token_info = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: None,
    };
    let staking_info = StakingInfoResponse {
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
        price: msg.price,
    };
    TOKEN_INFO.save(deps.storage, &query_token_info)?;
    STAKING_INFO.save(deps.storage, &staking_info)?;
    Ok(Response::default())
}

#[cw_serde]
pub enum ExecuteMsg {
    SetPrice(Uint128),
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SetPrice(price) => {
            let mut staking_info = STAKING_INFO.load(_deps.storage)?;
            staking_info.price = price;
            STAKING_INFO.save(_deps.storage, &staking_info)?;
        }
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: StakingDerivativeQueryMsg) -> StdResult<Binary> {
    match msg {
        StakingDerivativeQueryMsg::StakingInfo { .. } => {
            to_binary(&STAKING_INFO.load(deps.storage)?)
        }
        StakingDerivativeQueryMsg::TokenInfo {} => {
            let info = TOKEN_INFO.load(deps.storage)?;
            let resp = QueryAnswer::TokenInfo {
                name: info.name,
                symbol: info.symbol,
                decimals: info.decimals,
                total_supply: info.total_supply,
            };
            to_binary(&resp)
        }
    }
}
