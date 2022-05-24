use shade_oracles::{
    common::querier::query_token_info,
    common::{CanonicalContract, ResponseStatus, BLOCK_SIZE},
    storage::traits::SingletonStorable,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, QueryResult, StdResult, Storage, Uint128,
};
use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::{querier::{query_price}, QueryMsg, PriceResponse},
    router::querier::query_oracle,
    staking_derivative::shade::{
        querier::query_price as query_derivative_price,
        {ConfigResponse, InitMsg, HandleMsg, HandleStatusAnswer},
    },
};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub owner: CanonicalAddr,
    pub supported_symbol: String,
    pub underlying_symbol: String,
    pub router: CanonicalContract,
    pub staking_derivative: CanonicalContract,
    pub token_decimals: u8,
    pub enabled: bool,
}

impl SingletonStorable for State {
    fn namespace() -> Vec<u8> {
        b"config".to_vec()
    }
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let router: CanonicalContract = CanonicalContract {
        address: deps.api.canonical_address(&HumanAddr(msg.router.address))?,
        code_hash: msg.router.code_hash,
    };

    let staking_derivative: CanonicalContract = CanonicalContract {
        address: deps
            .api
            .canonical_address(&HumanAddr(msg.staking_derivative.address.clone()))?,
        code_hash: msg.staking_derivative.code_hash.clone(),
    };

    let token_decimals = query_token_info(&msg.staking_derivative, &deps.querier)?
        .token_info
        .decimals;

    let state: State = State {
        owner: deps.api.canonical_address(&HumanAddr(msg.owner))?,
        supported_symbol: msg.supported_symbol,
        underlying_symbol: msg.underlying_symbol,
        router,
        staking_derivative,
        token_decimals,
        enabled: true,
    };

    state.save_json(&mut deps.storage)?;

    Ok(InitResponse::default())
}

/* CONFIG UPDATE NEEDS TO BE FIXED */
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    pad_handle_result(
        // TODO: Add handle for set status
        match msg {
            HandleMsg::SetStatus { enabled } => try_update_status(deps, &env, enabled),
        },
        BLOCK_SIZE,
    )
}

fn try_update_status<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    enabled: bool,
) -> StdResult<HandleResponse> {
    CONFIG.load(&deps.storage)?.is_owner(env)?;
    let new_config = CONFIG.update(&mut deps.storage, |mut config| -> StdResult<_> {
        config.enabled = enabled;
        Ok(config)
    })?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleStatusAnswer { status: ResponseStatus::Success, enabled: new_config.enabled, })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => to_binary(&try_query_config(deps)?),
            QueryMsg::GetPrice { .. } => to_binary(&try_query_price(deps)?),
            QueryMsg::GetPrices { symbols } => todo!(),
        },
        BLOCK_SIZE,
    )
}

fn try_query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let state: State = State::new_json(&deps.storage)?;

    Ok(ConfigResponse {
        owner: deps.api.human_address(&state.owner)?.to_string(),
        router: state.router.as_human(&deps.api)?,
        staking_derivative: state.staking_derivative.as_human(&deps.api)?,
        supported_symbol: state.supported_symbol,
        underlying_symbol: state.underlying_symbol,
        enabled: state.enabled,
    })
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<PriceResponse> {
    let state: State = State::new_json(&deps.storage)?;

    let underlying_oracle = query_oracle(
        &state.router.as_human(&deps.api)?,
        &deps.querier,
        state.underlying_symbol,
    )?;

    // price of underlying asset to 10^18.
    let underlying_price = query_price(&underlying_oracle, &deps.querier, state.underlying_symbol)?;

    let staking_derivative_price = query_derivative_price(
        &state.staking_derivative.as_human(&deps.api)?,
        &deps.querier,
    )?;

    let staking_derivative_price_precision =
        Uint128(get_precision(state.token_decimals).clamp_u128()?);

    let price = underlying_price
        .rate
        .multiply_ratio(staking_derivative_price, staking_derivative_price_precision);

    let response = PriceResponse {
        rate: price,
        last_updated_base: underlying_price.last_updated_base,
        last_updated_quote: underlying_price.last_updated_quote,
    };
    Ok(response)
}
