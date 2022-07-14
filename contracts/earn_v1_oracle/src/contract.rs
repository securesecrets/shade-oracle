use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::{
        get_precision,
        BLOCK_SIZE, querier::query_token_info,
        CanonicalContract, Contract, ResponseStatus,
    },
    protocols::shade_earn_v1::{query_deposit_for_shares, query_generic_config},
};
use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Env, Extern, Response, Addr, InitResponse,
    Querier, StdResult<QueryResponse>, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use shade_oracles::{
    common::{query_price, PriceResponse, QueryMsg},
    earn::{ConfigResponse, HandleAnswer, ExecuteMsg, InstantiateMsg},
};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub owner: CanonicalAddr,
    pub deposit_token_oracle: CanonicalContract,
    pub strategy: CanonicalContract,
    pub deposit_token: CanonicalContract,
    pub share_token: CanonicalContract,
    pub deposit_token_decimals: u8,
    pub share_token_decimals: u8,
}

impl SingletonStorable for State {
    fn namespace() -> Vec<u8> {
        b"config".to_vec()
    }
}

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<InitResponse> {
    let underlying_oracle = msg.deposit_token_oracle.as_canonical(&deps.api)?;
    let config = query_generic_config(&msg.strategy, &deps.querier)?;
    let deposit_token = config.deposit_token.as_canonical(&deps.api)?;
    let share_token = config.share_token.as_canonical(&deps.api)?;
    let deposit_token_decimals = query_token_info(&config.deposit_token, &deps.querier)?
        .token_info
        .decimals;
    let share_token_decimals = query_token_info(&config.share_token, &deps.querier)?
        .token_info
        .decimals;

    let state: State = State {
        owner: deps.api.canonical_address(&Addr(msg.owner))?,
        deposit_token_oracle: underlying_oracle,
        deposit_token,
        share_token,
        strategy: msg.strategy.as_canonical(&deps.api)?,
        deposit_token_decimals,
        share_token_decimals,
    };

    state.save_json(&mut deps.storage)?;

    Ok(InitResponse::default())
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateConfig {
                owner,
                deposit_token_oracle,
                strategy,
            } => try_update_config(deps, env, owner, deposit_token_oracle, strategy),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config(
    deps: DepsMut,
    env: Env,
    owner: Option<String>,
    deposit_token_oracle: Option<Contract>,
    strategy: Option<Contract>,
) -> StdResult<Response> {
    let mut state: State = State::new_json(&deps.storage)?;

    if deps.api.canonical_address(&env.message.sender)? != state.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    if let Some(owner) = owner {
        state.owner = deps.api.canonical_address(&Addr(owner))?;
    }

    if let Some(deposit_token_oracle) = deposit_token_oracle {
        state.deposit_token_oracle = deposit_token_oracle.as_canonical(&deps.api)?;
    }

    if let Some(strategy) = strategy {
        let config = query_generic_config(&strategy, &deps.querier)?;
        let deposit_token_decimals = query_token_info(&config.deposit_token, &deps.querier)?
            .token_info
            .decimals;
        let share_token_decimals = query_token_info(&config.share_token, &deps.querier)?
            .token_info
            .decimals;

        state.strategy = strategy.as_canonical(&deps.api)?;
        state.deposit_token = config.deposit_token.as_canonical(&deps.api)?;
        state.share_token = config.share_token.as_canonical(&deps.api)?;
        state.deposit_token_decimals = deposit_token_decimals;
        state.share_token_decimals = share_token_decimals;
    }

    state.save_json(&mut deps.storage)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => to_binary(&try_query_config(deps)?),
            QueryMsg::GetPrice { .. } => to_binary(&try_query_price(deps)?),
        },
        BLOCK_SIZE,
    )
}

fn try_query_config(
    deps: Deps,
) -> StdResult<ConfigResponse> {
    let state: State = State::new_json(&deps.storage)?;

    Ok(ConfigResponse {
        owner: deps.api.human_address(&state.owner)?.to_string(),
        strategy: state.strategy.as_human(&deps.api)?,
        deposit_token_oracle: state.deposit_token_oracle.as_human(&deps.api)?,
        deposit_token: state.deposit_token.as_human(&deps.api)?,
        share_token: state.share_token.as_human(&deps.api)?,
    })
}

fn try_query_price(
    deps: Deps,
) -> StdResult<PriceResponse> {
    let state: State = State::new_json(&deps.storage)?;

    let deposits_for_one_share = Uint256::from(query_deposit_for_shares(
        &state.strategy.as_human(&deps.api)?,
        &deps.querier,
        Uint128(10u128.pow(state.share_token_decimals.into())),
    )?);

    let deposit_token_oracle_response = query_price(
        &state.deposit_token_oracle.as_human(&deps.api)?,
        &deps.querier,
    )?;

    let price_per_deposit_token = Uint256::from(deposit_token_oracle_response.rate);

    let price_per_share = deposits_for_one_share
        .multiply_ratio(
            price_per_deposit_token,
            get_precision(state.deposit_token_decimals),
        )?
        .clamp_u128()?;

    let response = PriceResponse {
        rate: Uint128(price_per_share),
        last_updated_base: deposit_token_oracle_response.last_updated_base,
        last_updated_quote: deposit_token_oracle_response.last_updated_quote,
    };
    Ok(response)
}
