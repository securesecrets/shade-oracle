use shade_oracles::{
    common::{
        Contract, ResponseStatus, 
        OraclePrice, QueryMsg, BLOCK_SIZE, is_disabled
    }, 
    band::{
        ReferenceData, BandQuery,
        proxy::{
            HandleMsg, HandleAnswer,
            Config, InitMsg,
        }
    },
    storage::Item,
};
use shade_admin::admin::{QueryMsg as AdminQueryMsg, ValidateAdminPermissionResponse};
use cosmwasm_std::{
    to_binary, Binary, Api, Env, 
    Extern, HandleResponse, 
    InitResponse, Querier, QueryResult,
    StdError, StdResult, Storage
};
use cosmwasm_math_compat::Uint128;
use secret_toolkit::utils::{pad_handle_result, pad_query_result, Query};

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
    
    let resp: ValidateAdminPermissionResponse = AdminQueryMsg::ValidateAdminPermission { contract_address: env.contract.address.to_string(), admin_address: env.message.sender.to_string() }.query(&deps.querier, config.admin_auth.code_hash.clone(), config.admin_auth.address)?;
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

    if key == "SHD" {
        return to_binary(&OraclePrice::new(key, ReferenceData { rate: Uint128::from(13450000000000000000u128), last_updated_base: 1654019032, last_updated_quote: 1654019032 }))
    }

    let band_response: ReferenceData = BandQuery::GetReferenceData {
        base_symbol: key.clone(),
        quote_symbol: config.quote_symbol,
    }
    .query(
        &deps.querier,
        config.band.code_hash,
        config.band.address,
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

    let band_response: Vec<ReferenceData> = BandQuery::GetReferenceDataBulk {
        base_symbols: keys.clone(),
        quote_symbols,
    }.query(&deps.querier, config.band.code_hash, config.band.address)?;

    let mut prices: Vec<OraclePrice> = vec![];
    for (index, key) in keys.iter().enumerate() {
        prices.push(OraclePrice::new(key.to_string(), band_response[index].clone()));
    };

    to_binary(&prices)
}
