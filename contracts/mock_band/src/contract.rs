use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_oracles::band::{HandleAnswer, HandleMsg, InitMsg, ReferenceData};
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, Storage,
};
use cosmwasm_math_compat::Uint128;
use shade_oracles::storage::Map;
use shade_oracles::{
    common::ResponseStatus,
};

const MOCK_DATA: Map<(String, String), ReferenceData> = Map::new("price-data");

pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateSymbolPrice {
            base_symbol,
            quote_symbol,
            rate,
            last_updated,
        } => update_symbol_price(
            deps,
            env,
            base_symbol,
            quote_symbol,
            rate,
            last_updated,
        ),
    }
}

pub fn update_symbol_price<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    base_symbol: String,
    quote_symbol: String,
    rate: Uint128,
    last_updated: Option<u64>,
) -> StdResult<HandleResponse> {

    MOCK_DATA.save(&mut deps.storage, (base_symbol, quote_symbol), &ReferenceData {
        rate,
        last_updated_base: last_updated.unwrap_or(0),
        last_updated_quote: last_updated.unwrap_or(0),
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateSymbolPrice { status: ResponseStatus::Success })?),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
    GetReferenceDataBulk {
        base_symbols: Vec<String>,
        quote_symbols: Vec<String>,
    },
}
pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetReferenceData {
            base_symbol,
            quote_symbol,
        } => query_saved_band_data(deps, base_symbol, quote_symbol),
        QueryMsg::GetReferenceDataBulk {
            base_symbols,
            quote_symbols,
        } => {
            let mut results = vec![];

            for (base, quote) in base_symbols.iter().zip(quote_symbols) {
                results.push(MOCK_DATA.load(&deps.storage, (base.to_string(), quote.to_string()))?);
            }
            to_binary(&results)
        }
    }
}

fn query_saved_band_data<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    base_symbol: String,
    quote_symbol: String,
) -> StdResult<Binary> {
    /*
    let data: ReferenceData = MOCK_DATA.load(&deps.storage, (base_symbol.clone(), quote_symbol.clone()))?;
    assert_eq!(data.rate, Uint128::zero(), "MOCK BAND REF");
    */
    to_binary(&MOCK_DATA.load(&deps.storage, (base_symbol, quote_symbol))?)
}
