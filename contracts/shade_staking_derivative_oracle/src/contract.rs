use shade_oracles::{
    common::{
        querier::{verify_admin, query_token_info, query_band_price},
        HandleAnswer, HandleMsg, QueryMsg,
        ResponseStatus, BLOCK_SIZE, throw_unsupported_symbol_error, get_precision, OraclePrice
    }, 
    band::ReferenceData, storage::Item,
    staking_derivative::shade::{
        querier::query_derivative_price,
        {Config, InitMsg},
    },
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, InitResponse,
    Querier, QueryResult, StdResult, Storage, Binary, StdError,
};

const CONFIG: Item<Config> = Item::new("config");
const TOKEN_DECIMALS: Item<u8> = Item::new("token_decimals");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let token_decimals = query_token_info(&msg.staking_derivative, &deps.querier)?
        .token_info
        .decimals;

    let config = Config {
        supported_key: msg.supported_key,
        underlying_symbol: msg.underlying_symbol,
        router: msg.router,
        staking_derivative: msg.staking_derivative,
        enabled: true,
    };

    TOKEN_DECIMALS.save(&mut deps.storage, &token_decimals)?;
    CONFIG.save(&mut deps.storage, &config)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    pad_handle_result(
        match msg {
            HandleMsg::UpdateConfig { enabled } => try_update_config(deps, &env, enabled),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    enabled: bool,
) -> StdResult<HandleResponse> {
    let config = CONFIG.load(&deps.storage)?;
    verify_admin(&config.router, &deps.querier, env.message.sender.clone())?;
    CONFIG.update(&mut deps.storage, |mut config| -> StdResult<_> {
        config.enabled = enabled;
        Ok(config)
    })?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig { status: ResponseStatus::Success })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => to_binary(&CONFIG.load(&deps.storage)?),
            QueryMsg::GetPrice { key } => try_query_price(deps, key),
            QueryMsg::GetPrices { .. } => Err(StdError::generic_err("Unsupported method.")),
        },
        BLOCK_SIZE,
    )
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: String,
) -> StdResult<Binary> {
    let config = CONFIG.load(&deps.storage)?;
    let token_decimals = TOKEN_DECIMALS.load(&deps.storage)?;

    if key != config.supported_key {
        return Err(throw_unsupported_symbol_error(key));
    }

    // price of underlying asset to 10^18.
    let underlying_price = query_band_price(&config.router, &deps.querier, config.underlying_symbol)?;

    let staking_derivative_price = query_derivative_price(
        &config.staking_derivative,
        &deps.querier,
    )?;

    let staking_derivative_price_precision =
        get_precision(token_decimals);

    let price = underlying_price
        .data
        .rate
        .multiply_ratio(staking_derivative_price, staking_derivative_price_precision);

    let response = ReferenceData {
        rate: price,
        last_updated_base: underlying_price.data.last_updated_base,
        last_updated_quote: underlying_price.data.last_updated_quote,
    };
    to_binary(&OraclePrice::new(key, response))
}
