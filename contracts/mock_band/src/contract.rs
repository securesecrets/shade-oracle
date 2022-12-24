use cosmwasm_std::{entry_point, Storage};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use shade_oracles::core::{pad_query_result, ResponseStatus};
use shade_oracles::interfaces::band::{
    ExecuteAnswer, ExecuteMsg, InstantiateMsg, MockPrice, QueryMsg, ReferenceData,
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
        ExecuteMsg::SetPrice { price } => set_price(deps, env, price),
        ExecuteMsg::SetPrices { prices } => set_prices(deps, env, prices),
    }
}

pub fn set_mock_price(storage: &mut dyn Storage, now: u64, price: MockPrice) -> StdResult<()> {
    MOCK_DATA.save(
        storage,
        (price.base_symbol, price.quote_symbol),
        &ReferenceData {
            rate: price.rate,
            last_updated_base: price.last_updated.unwrap_or_else(|| now),
            last_updated_quote: price.last_updated.unwrap_or_else(|| now),
        },
    )
}

pub fn set_price(deps: DepsMut, env: Env, price: MockPrice) -> StdResult<Response> {
    set_mock_price(deps.storage, env.block.time.seconds(), price)?;
    let data = to_binary(&ExecuteAnswer::SetPrice {
        status: ResponseStatus::Success,
    })?;

    Ok(Response::new().set_data(data))
}

pub fn set_prices(deps: DepsMut, env: Env, prices: Vec<MockPrice>) -> StdResult<Response> {
    for price in prices {
        set_mock_price(deps.storage, env.block.time.seconds(), price)?;
    }
    let data = to_binary(&ExecuteAnswer::SetPrices {
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
    to_binary(&MOCK_DATA.load(deps.storage, (base_symbol, quote_symbol))?)
}
