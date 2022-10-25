use cosmwasm_std::{entry_point, Uint128};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use shade_oracles::core::{pad_query_result, ResponseStatus};
use shade_oracles::interfaces::band::{
    ExecuteMsg, HandleAnswer, InstantiateMsg, QueryMsg, ReferenceData,
};
use shade_oracles::ssp::Map;
use shade_oracles::BLOCK_SIZE;

const MOCK_DATA: Map<(String, String), ReferenceData> = Map::new("price-data");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let now = env.block.time.seconds();
    for (base, quote, rate) in msg.initial_prices {
        MOCK_DATA.save(
            deps.storage,
            (base, quote),
            &ReferenceData {
                rate,
                last_updated_base: now,
                last_updated_quote: now,
            },
        )?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
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
    env: Env,
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
            last_updated_base: last_updated.unwrap_or_else(|| env.block.time.seconds()),
            last_updated_quote: last_updated.unwrap_or_else(|| env.block.time.seconds()),
        },
    )?;

    let data = to_binary(&HandleAnswer::UpdateSymbolPrice {
        status: ResponseStatus::Success,
    })?;

    Ok(Response::new().set_data(data))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
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
                    results
                        .push(MOCK_DATA.load(deps.storage, (base.to_string(), quote.to_string()))?);
                }
                to_binary(&results)
            }
        },
        BLOCK_SIZE,
    )
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
