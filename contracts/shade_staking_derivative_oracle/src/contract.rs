use cosmwasm_std::{
    to_binary, MessageInfo, Binary, Env, Deps, Response, Addr, DepsMut,
    QueryRequest, StdError, StdResult, QueryResponse, WasmQuery, entry_point,
};
use shade_oracles::{
    pad_handle_result, pad_query_result, ResponseStatus, Contract, BLOCK_SIZE,
    interfaces::band::ReferenceData,
    common::{
        is_disabled,
        querier::{query_prices, query_token_info, verify_admin, query_band_price},
        throw_unsupported_symbol_error, HandleAnswer, ExecuteMsg, OraclePrice, OracleQuery, get_precision
    },
    interfaces::staking_derivative::shade::{
        querier::query_derivative_price,
        {Config, InstantiateMsg},
    },
    storage::Item,
};

const CONFIG: Item<Config> = Item::new("config");
const TOKEN_DECIMALS: Item<u8> = Item::new("token_decimals");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let token_decimals = query_token_info(&msg.staking_derivative, &deps.querier)?
        .decimals;

    let config = Config {
        supported_key: msg.supported_key,
        underlying_symbol: msg.underlying_symbol,
        router: msg.router,
        staking_derivative: msg.staking_derivative,
        enabled: true,
    };

    TOKEN_DECIMALS.save(deps.storage, &token_decimals)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    verify_admin(&config.router, deps.as_ref(), info.sender.clone())?;

    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateConfig { enabled } => try_update_config(deps, enabled),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config(
    deps: DepsMut,
    enabled: bool,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.enabled = enabled;
        Ok(config)
    })?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateConfig {
        status: ResponseStatus::Success,
    })?))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: OracleQuery) -> StdResult<QueryResponse> {
    pad_query_result(
        match msg {
            OracleQuery::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
            OracleQuery::GetPrice { key } => try_query_price(deps, key),
            OracleQuery::GetPrices { .. } => Err(StdError::generic_err("Unsupported method.")),
        },
        BLOCK_SIZE,
    )
}

fn try_query_price(
    deps: Deps,
    key: String,
) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let token_decimals = TOKEN_DECIMALS.load(deps.storage)?;

    if key != config.supported_key {
        return Err(throw_unsupported_symbol_error(key));
    }

    // price of underlying asset to 10^18.
    let underlying_price =
        query_band_price(&config.router, &deps.querier, config.underlying_symbol)?;

    let staking_derivative_price =
        query_derivative_price(deps, &config.staking_derivative)?;

    let staking_derivative_price_precision = get_precision(token_decimals);

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
