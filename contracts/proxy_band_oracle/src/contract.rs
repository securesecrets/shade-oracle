use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::{
        Contract, ResponseStatus, CommonOracleConfig, 
        OraclePrice, QueryMsg, BLOCK_SIZE
    }, 
    band::{
        ReferenceData, BandQuery,
        proxy::{
            HandleMsg, HandleAnswer,
            Config, InitMsg,
        }
    },
    storage::Item, router::querier::verify_admin,
};
use cosmwasm_std::{
    to_binary, Binary, Api, Env, 
    Extern, HandleResponse, HumanAddr, 
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
    config.is_owner(&env)?;

    CONFIG.update(&mut deps.storage, |mut state| -> StdResult<_> {
        state.band = band.unwrap_or(state.band);
        state.quote_symbol = quote_symbol.unwrap_or(state.quote_symbol);
        Ok(state)
    })?;

    CONFIG.update(&mut deps.storage, |mut config| -> StdResult<_> {
        config.owner = owner.unwrap_or(config.owner);
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
        QueryMsg::GetConfig {} => try_query_config(deps),
        QueryMsg::GetPrice { key } => try_query_price(deps, key),
        QueryMsg::GetPrices { keys } => try_query_prices(deps, keys),
    };
    pad_query_result(response, BLOCK_SIZE)
}

fn try_query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let state = STATE.load(&deps.storage)?;
    let common = CONFIG.load(&deps.storage)?;

    to_binary(&ConfigResponse {
        owner: common.owner,
        band: state.band,
        quote_symbol: state.quote_symbol,
        enabled: common.enabled,
    })
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: String,
) -> StdResult<Binary> {
    CONFIG.load(&deps.storage)?.is_enabled()?;

    let state = STATE.load(&deps.storage)?;

    if key == "SHD" {
        return to_binary(&OraclePrice::new(key, ReferenceData { rate: Uint128::from(13450000000000000000u128), last_updated_base: 1654019032, last_updated_quote: 1654019032 }))
    }

    let band_response: ReferenceData = BandQuery::GetReferenceData {
        base_symbol: key.clone(),
        quote_symbol: state.quote_symbol,
    }
    .query(
        &deps.querier,
        state.band.code_hash,
        state.band.address,
    )?;

    to_binary(&OraclePrice::new(key, band_response))
}

fn try_query_prices<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    keys: Vec<String>,
) -> StdResult<Binary> {
    CONFIG.load(&deps.storage)?.is_enabled()?;

    let state = STATE.load(&deps.storage)?;

    let quote_symbols = vec![state.quote_symbol; keys.len()];

    let band_response: Vec<ReferenceData> = BandQuery::GetReferenceDataBulk {
        base_symbols: keys.clone(),
        quote_symbols,
    }.query(&deps.querier, state.band.code_hash, state.band.address)?;

    let mut prices: Vec<OraclePrice> = vec![];
    for (index, key) in keys.iter().enumerate() {
        prices.push(OraclePrice::new(key.to_string(), band_response[index].clone()));
    };

    to_binary(&prices)
}
