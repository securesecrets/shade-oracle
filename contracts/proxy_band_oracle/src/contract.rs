use cosmwasm_std::{Uint128, QueryResponse, entry_point};
use cosmwasm_std::{
    to_binary, DepsMut, Binary, Env, Deps, Response, MessageInfo, 
   StdError, StdResult,
};
use shade_oracles::common::querier::verify_admin;
use shade_oracles::validate_admin;
use shade_oracles::{
    pad_handle_result, pad_query_result, Contract, ResponseStatus, BLOCK_SIZE,
    interfaces::band::{
        proxy::{Config, ExecuteMsg, InstantiateMsg},
        reference_data, reference_data_bulk, ReferenceData,
    },
    common::{is_disabled, HandleAnswer, OraclePrice, OracleQuery},
    storage::Item,
};

const CONFIG: Item<Config> = Item::new("config");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        admin_auth: msg.admin_auth,
        band: msg.band,
        quote_symbol: msg.quote_symbol,
        enabled: true,
    };

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
    validate_admin(&deps.querier, env.contract.address.to_string(), info.sender.to_string(), &config.admin_auth)?;

    let response: Result<Response, StdError> = match msg {
        ExecuteMsg::UpdateConfig {
            band,
            quote_symbol,
            enabled,
            admin_auth,
        } => try_update_config(deps, band, quote_symbol, enabled, admin_auth),
    };
    pad_handle_result(response, BLOCK_SIZE)
}

fn try_update_config(
    deps: DepsMut,
    band: Option<Contract>,
    quote_symbol: Option<String>,
    enabled: Option<bool>,
    admin_auth: Option<Contract>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.band = band.unwrap_or(config.band);
        config.quote_symbol = quote_symbol.unwrap_or(config.quote_symbol);
        config.enabled = enabled.unwrap_or(config.enabled);
        config.admin_auth = admin_auth.unwrap_or(config.admin_auth);
        Ok(config)
    })?;

    let data = to_binary(&HandleAnswer::UpdateConfig {
        status: ResponseStatus::Success,
    })?;

    Ok(Response::new().set_data(data))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: OracleQuery) -> StdResult<QueryResponse> {
    let response = match msg {
        OracleQuery::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
        OracleQuery::GetPrice { key } => try_query_price(deps, key),
        OracleQuery::GetPrices { keys } => try_query_prices(deps, keys),
    };
    pad_query_result(response, BLOCK_SIZE)
}

fn try_query_price(
    deps: Deps,
    key: String,
) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    is_disabled(config.enabled)?;

    if key == "SHD" {
        return to_binary(&OraclePrice::new(
            key,
            ReferenceData {
                rate: Uint128::from(13450000000000000000u128),
                last_updated_base: 1654019032,
                last_updated_quote: 1654019032,
            },
        ));
    }

    let band_response = reference_data(
        &deps.querier,
        key.clone(),
        config.quote_symbol.clone(),
        config.band,
    )?;

    to_binary(&OraclePrice::new(key, band_response))
}

fn try_query_prices(
    deps: Deps,
    keys: Vec<String>,
) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    is_disabled(config.enabled)?;

    let quote_symbols = vec![config.quote_symbol; keys.len()];

    let band_response =
        reference_data_bulk(&deps.querier, keys.clone(), quote_symbols, config.band)?;

    let mut prices: Vec<OraclePrice> = vec![];
    for (index, key) in keys.iter().enumerate() {
        prices.push(OraclePrice::new(
            key.to_string(),
            band_response[index].clone(),
        ));
    }

    to_binary(&prices)
}
