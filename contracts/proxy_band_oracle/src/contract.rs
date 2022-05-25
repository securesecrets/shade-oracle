use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::{Contract, ResponseStatus, CommonOracleConfig, HandleStatusAnswer, OraclePrice, QueryMsg, BLOCK_SIZE}, band::{ReferenceData, proxy::{HandleMsg, HandleAnswer}}, storage::Item,
};
use cosmwasm_std::{to_binary, Binary, Api, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryResult,
    StdError, StdResult, Storage,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result, Query};
use shade_oracles::{
    band::{
        proxy::{ConfigResponse, InitMsg},
        BandQuery,
    },
};

/// state of the auction
#[derive(Serialize, Deserialize)]
pub struct State {
    pub band: Contract,
    /// Price in which requests will be quoted in
    pub quote_symbol: String,
}

const CONFIG: Item<CommonOracleConfig> = Item::new("config");
const STATE: Item<State> = Item::new("band-state");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let state = State {
        band: msg.band,
        quote_symbol: msg.quote_symbol,
    };

    let common = CommonOracleConfig {
        owner: msg.owner,
        enabled: true,
    };

    STATE.save(&mut deps.storage, &state)?;
    CONFIG.save(&mut deps.storage, &common)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let response: Result<HandleResponse, StdError> = match msg {
        HandleMsg::SetStatus { enabled, } => try_update_status(deps, &env, enabled),
        HandleMsg::UpdateConfig {
            owner,
            band,
            quote_symbol,
        } => try_update_config(deps, env, owner, band, quote_symbol),
    };
    pad_handle_result(response, BLOCK_SIZE)
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

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
    band: Option<Contract>,
    quote_symbol: Option<String>,
) -> StdResult<HandleResponse> {
    let config = CONFIG.load(&deps.storage)?;
    config.is_owner(&env)?;

    STATE.update(&mut deps.storage, |mut state| -> StdResult<_> {
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
        QueryMsg::GetPrice { symbol } => try_query_price(deps, symbol),
        QueryMsg::GetPrices { symbols } => try_query_prices(deps, symbols),
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
    symbol: String,
) -> StdResult<Binary> {
    CONFIG.load(&deps.storage)?.is_enabled()?;

    let state = STATE.load(&deps.storage)?;

    let band_response: ReferenceData = BandQuery::GetReferenceData {
        base_symbol: symbol.clone(),
        quote_symbol: state.quote_symbol,
    }
    .query(
        &deps.querier,
        state.band.code_hash,
        state.band.address,
    )?;

    to_binary(&OraclePrice::new(symbol, band_response))
}

fn try_query_prices<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbols: Vec<String>,
) -> StdResult<Binary> {
    CONFIG.load(&deps.storage)?.is_enabled()?;

    let state = STATE.load(&deps.storage)?;

    let quote_symbols = vec![state.quote_symbol; symbols.len()];

    let band_response: Vec<ReferenceData> = BandQuery::GetReferenceDataBulk {
        base_symbols: symbols.clone(),
        quote_symbols,
    }.query(&deps.querier, state.band.code_hash, state.band.address)?;

    let mut prices: Vec<OraclePrice> = vec![];
    for (index, symbol) in symbols.iter().enumerate() {
        prices.push(OraclePrice::new(symbol.to_string(), band_response[index].clone()));
    };

    to_binary(&prices)
}
