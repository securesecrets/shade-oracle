use shade_oracles::{
    common::{querier::query_token_info, CommonOracleConfig},
    common::{ResponseStatus, BLOCK_SIZE, Contract}, band::ReferenceData, storage::Item,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, InitResponse,
    Querier, QueryResult, StdResult, Storage, Uint128, Binary, StdError,
};
use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::{get_precision, querier::{query_price}, throw_unsupported_symbol_error, QueryMsg, OraclePrice},
    router::querier::query_oracle,
    staking_derivative::shade::{
        querier::query_price as query_derivative_price,
        {ConfigResponse, InitMsg, HandleMsg, HandleStatusAnswer},
    },
};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub supported_symbol: String,
    pub underlying_symbol: String,
    pub router: Contract,
    pub staking_derivative: Contract,
    pub token_decimals: u8,
}

const CONFIG: Item<CommonOracleConfig> = Item::new("config");
const STATE: Item<State> = Item::new("state");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let token_decimals = query_token_info(&msg.staking_derivative, &deps.querier)?
        .token_info
        .decimals;

    let state: State = State {
        supported_symbol: msg.supported_symbol,
        underlying_symbol: msg.underlying_symbol,
        router: msg.router,
        staking_derivative: msg.staking_derivative,
        token_decimals,
    };

    let config = CommonOracleConfig { owner: msg.owner, enabled: true };

    STATE.save(&mut deps.storage, &state)?;
    CONFIG.save(&mut deps.storage, &config)?;

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
            QueryMsg::GetPrice { symbol } => try_query_price(deps, symbol),
            QueryMsg::GetPrices { .. } => Err(StdError::generic_err("Unsupported method.")),
        },
        BLOCK_SIZE,
    )
}

fn try_query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let state = STATE.load(&deps.storage)?;
    let config = CONFIG.load(&deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        router: state.router,
        staking_derivative: state.staking_derivative,
        supported_symbol: state.supported_symbol,
        underlying_symbol: state.underlying_symbol,
        enabled: config.enabled,
    })
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
) -> StdResult<Binary> {
    let state = STATE.load(&deps.storage)?;

    if symbol != state.supported_symbol {
        return Err(throw_unsupported_symbol_error(symbol));
    }

    let underlying_oracle = query_oracle(
        &state.router,
        &deps.querier,
        state.underlying_symbol.clone(),
    )?;

    // price of underlying asset to 10^18.
    let underlying_price = query_price(&underlying_oracle, &deps.querier, state.underlying_symbol)?;

    let staking_derivative_price = query_derivative_price(
        &state.staking_derivative,
        &deps.querier,
    )?;

    let staking_derivative_price_precision =
        Uint128(get_precision(state.token_decimals).clamp_u128()?);

    let price = underlying_price
        .price
        .rate
        .multiply_ratio(staking_derivative_price, staking_derivative_price_precision);

    let response = ReferenceData {
        rate: price,
        last_updated_base: underlying_price.price.last_updated_base,
        last_updated_quote: underlying_price.price.last_updated_quote,
    };
    to_binary(&OraclePrice::new(symbol, response))
}
