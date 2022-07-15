use cosmwasm_std::{
    to_binary, Api, Binary, Env, Deps, Response,  Querier, 
   StdError, StdResult, Storage,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use shade_oracles::{
    band::ReferenceData,
    common::{
        get_precision,
        querier::{query_band_price, query_token_info, verify_admin},
        throw_unsupported_symbol_error, HandleAnswer, ExecuteMsg, OraclePrice, QueryMsg,
        ResponseStatus, BLOCK_SIZE,
    },
    staking_derivative::shade::{
        querier::query_derivative_price,
        {Config, InstantiateMsg},
    },
    storage::Item,
};

const CONFIG: Item<Config> = Item::new("config");
const TOKEN_DECIMALS: Item<u8> = Item::new("token_decimals");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
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

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateConfig { enabled } => try_update_config(deps, &env, enabled),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config(
    deps: DepsMut,
    env: &Env,
    enabled: bool,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    verify_admin(&config.router, &deps.querier, env.message.sender.clone())?;
    CONFIG.update(&mut deps.storage, |mut config| -> StdResult<_> {
        config.enabled = enabled;
        Ok(config)
    })?;
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
            QueryMsg::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
            QueryMsg::GetPrice { key } => try_query_price(deps, key),
            QueryMsg::GetPrices { .. } => Err(StdError::generic_err("Unsupported method.")),
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
        query_derivative_price(&config.staking_derivative, &deps.querier)?;

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
