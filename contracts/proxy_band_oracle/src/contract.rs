use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, QueryResult,
    StdError, StdResult, Storage,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result, Query};
use shade_admin::admin::{QueryMsg as AdminQueryMsg, ValidateAdminPermissionResponse};
use shade_oracles::{
    band::{
        proxy::{Config, HandleAnswer, HandleMsg, InitMsg},
        reference_data, reference_data_bulk, BandQuery, ReferenceData,
    },
    common::{is_disabled, Contract, OraclePrice, QueryMsg, ResponseStatus, BLOCK_SIZE},
    storage::Item,
};

const CONFIG: Item<Config> = Item::new("config");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config {
        admin_auth: msg.admin_auth,
        band: msg.band,
        quote_symbol: msg.quote_symbol,
        enabled: true,
    };

    CONFIG.save(&mut deps.storage, &config)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let response: Result<HandleResponse, StdError> = match msg {
        HandleMsg::UpdateConfig {
            band,
            quote_symbol,
            enabled,
            admin_auth,
        } => try_update_config(deps, env, band, quote_symbol, enabled, admin_auth),
    };
    pad_handle_result(response, BLOCK_SIZE)
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    band: Option<Contract>,
    quote_symbol: Option<String>,
    enabled: Option<bool>,
    admin_auth: Option<Contract>,
) -> StdResult<HandleResponse> {
    let config = CONFIG.load(&deps.storage)?;

    let resp: ValidateAdminPermissionResponse = AdminQueryMsg::ValidateAdminPermission {
        contract_address: env.contract.address.to_string(),
        admin_address: env.message.sender.to_string(),
    }
    .query(
        &deps.querier,
        config.admin_auth.code_hash.clone(),
        config.admin_auth.address,
    )?;
    if resp.error_msg.is_some() {
        return Err(StdError::unauthorized());
    }

    CONFIG.update(&mut deps.storage, |mut config| -> StdResult<_> {
        config.band = band.unwrap_or(config.band);
        config.quote_symbol = quote_symbol.unwrap_or(config.quote_symbol);
        config.enabled = enabled.unwrap_or(config.enabled);
        config.admin_auth = admin_auth.unwrap_or(config.admin_auth);
        Ok(config)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    let response = match msg {
        QueryMsg::GetConfig {} => to_binary(&CONFIG.load(&deps.storage)?),
        QueryMsg::GetPrice { key } => try_query_price(deps, key),
        QueryMsg::GetPrices { keys } => try_query_prices(deps, keys),
    };
    pad_query_result(response, BLOCK_SIZE)
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: String,
) -> StdResult<Binary> {
    let config = CONFIG.load(&deps.storage)?;
    is_disabled(config.enabled)?;

    let band_response = reference_data(
        &deps.querier,
        key.clone(),
        config.quote_symbol.clone(),
        config.band,
    )?;

    to_binary(&OraclePrice::new(key, band_response))
}

fn try_query_prices<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    keys: Vec<String>,
) -> StdResult<Binary> {
    let config = CONFIG.load(&deps.storage)?;
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
