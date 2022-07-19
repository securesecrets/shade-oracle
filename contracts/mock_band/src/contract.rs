use cosmwasm_std::{Uint128, entry_point};
use cosmwasm_std::{
    to_binary, Binary, Env, Deps, Response, StdResult, DepsMut, MessageInfo
};

use shade_oracles::interfaces::band::{QueryMsg, HandleAnswer, ExecuteMsg, InstantiateMsg, ReferenceData};
use shade_oracles::core::{ResponseStatus};
use shade_oracles::storage::Map;

const MOCK_DATA: Map<(String, String), ReferenceData> = Map::new("price-data");

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateSymbolPrice {
            base_symbol,
            quote_symbol,
            rate,
            last_updated,
        } => update_symbol_price(deps, env, base_symbol, quote_symbol, rate, last_updated),
    }
}

pub fn update_symbol_price(
    deps: DepsMut,
    _env: Env,
    base_symbol: String,
    quote_symbol: String,
    rate: Uint128,
    last_updated: Option<u64>,
) -> StdResult<Response> {
    MOCK_DATA.save(
        deps.storage,
        (base_symbol, quote_symbol),
        &ReferenceData {
            rate,
            last_updated_base: last_updated.unwrap_or(0),
            last_updated_quote: last_updated.unwrap_or(0),
        },
    )?;

    let data = to_binary(&HandleAnswer::UpdateSymbolPrice {
        status: ResponseStatus::Success,
    })?;

    Ok(Response::new().set_data(data))
}

#[entry_point]
pub fn query(
    deps: Deps,
    _env: Env,
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
                results.push(MOCK_DATA.load(deps.storage, (base.to_string(), quote.to_string()))?);
            }
            to_binary(&results)
        }
    }
}

fn query_saved_band_data(
    deps: Deps,
    base_symbol: String,
    quote_symbol: String,
) -> StdResult<Binary> {
    /*
    let data: ReferenceData = MOCK_DATA.load(deps.storage, (base_symbol.clone(), quote_symbol.clone()))?;
    assert_eq!(data.rate, Uint128::zero(), "MOCK BAND REF");
    */
    to_binary(&MOCK_DATA.load(deps.storage, (base_symbol, quote_symbol))?)
}
