use cosmwasm_std::{Uint128, Uint512, DepsMut, MessageInfo, QueryResponse, entry_point};
use cosmwasm_std::{
    to_binary, Env, Deps, Response, StdError,
    StdResult,
};
use std::{cmp::min, collections::HashMap};

use shade_oracles::{
    pad_handle_result, pad_query_result, Contract, BLOCK_SIZE, ResponseStatus,
    interfaces::
    {
        band::ReferenceData,
        common::{
        querier::{query_band_prices, query_prices, verify_admin},
        OraclePrice},
        index_oracle::{Config, HandleAnswer, ExecuteMsg, InstantiateMsg, QueryAnswer, QueryMsg},
    },
    storage::Item,
};

const CONFIG: Item<Config> = Item::new("config");
const SYMBOL: Item<String> = Item::new("symbol");
// (symbol, weight, constant)
const BASKET: Item<Vec<(String, Uint128, Uint128)>> = Item::new("basket");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        router: msg.router,
        enabled: true,
        only_band: msg.only_band,
    };

    CONFIG.save(deps.storage, &config)?;
    SYMBOL.save(deps.storage, &msg.symbol)?;

    if msg.basket.is_empty() {
        return Err(StdError::generic_err("Basket cannot be empty"));
    }

    let symbols: Vec<String> = msg.basket.clone().into_iter().map(|(sym, _)| sym).collect();

    if symbols.contains(&msg.symbol) {
        return Err(StdError::generic_err(format!(
            "Recursive symbol {}",
            msg.symbol
        )));
    }

    let weight_sum: u128 = msg.basket.clone().into_iter().map(|(_, w)| w.u128()).sum();
    if weight_sum != 10u128.pow(18) {
        return Err(StdError::generic_err(format!(
            "Weights must add to 100%, {}",
            weight_sum
        )));
    }

    let prices = fetch_prices(deps.as_ref(), &config, symbols)?;
    let constants = build_constants(msg.basket.clone(), prices, msg.target);

    let mut full_basket: Vec<(String, Uint128, Uint128)> = msg
        .basket
        .into_iter()
        .map(|(sym, w)| (sym.clone(), w, constants[&sym]))
        .collect();
    full_basket.sort();

    BASKET.save(deps.storage, &full_basket)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    verify_admin(&config.router, deps.as_ref(), info.sender)?;
    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateConfig {
                router,
                enabled,
                only_band,
            } => try_update_config(deps, router, enabled, only_band),
            ExecuteMsg::ModBasket { basket, .. } => mod_basket(deps, basket),
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => to_binary(&try_query_config(deps)?),
            /* add 'symbol' so we can error if its the wrong oracle
             * Prevents router failure from causing economic failure
             */
            QueryMsg::GetPrice { key, .. } => to_binary(&try_query_price(deps, key)?),
            QueryMsg::GetPrices { keys } => {
                let mut prices = vec![];
                for key in keys {
                    prices.push(try_query_price(deps, key)?);
                }
                to_binary(&prices)
            }
            QueryMsg::Basket {} => to_binary(&QueryAnswer::Basket {
                basket: BASKET.load(deps.storage)?,
            }),
        },
        BLOCK_SIZE,
    )
}

fn build_constants(
    weights: Vec<(String, Uint128)>,
    prices: HashMap<String, ReferenceData>,
    target: Uint128,
) -> HashMap<String, Uint128> {
    let mut constants: HashMap<String, Uint128> = HashMap::new();

    for (sym, weight) in weights {
        constants.insert(
            sym.clone(),
            weight.multiply_ratio(target, prices[&sym].rate),
        );
    }
    constants
}

fn eval_index(
    prices: HashMap<String, ReferenceData>,
    basket: Vec<(String, Uint128, Uint128)>,
) -> ReferenceData {
    let mut index_price = Uint512::zero();
    let mut last_updated_base = 0u64;
    let mut last_updated_quote = 0u64;

    for (sym, _, constant) in basket {
        index_price += Uint512::from(prices[&sym].rate.u128()) * Uint512::from(constant.u128())
            / Uint512::from(10u128.pow(18));
        last_updated_base = min(last_updated_base, prices[&sym].last_updated_base);
        last_updated_quote = min(last_updated_quote, prices[&sym].last_updated_quote);
    }
    ReferenceData {
        rate: Uint128::try_from(index_price).ok().unwrap(),
        last_updated_base,
        last_updated_quote,
    }
}

fn fetch_prices(
    deps: Deps,
    config: &Config,
    symbols: Vec<String>,
) -> StdResult<HashMap<String, ReferenceData>> {
    let mut price_data = HashMap::new();
    let prices_resp = if config.only_band {
        query_band_prices(&config.router, &deps.querier, symbols.clone())
    } else {
        query_prices(&config.router, &deps.querier, symbols.clone())
    };
    match prices_resp {
        Ok(prices) => {
            for oracle_price in prices {
                price_data.insert(oracle_price.key.clone(), oracle_price.data);
            }
        }
        Err(e) => {
            return Err(StdError::generic_err(format!(
                "Failed to query {} from router {}, '{}'",
                symbols
                    .iter()
                    .map(|sym| sym.to_string() + ",")
                    .collect::<String>(),
                config.router.address.as_str(),
                e
            )))
        }
    }
    Ok(price_data)
}

fn try_update_config(
    deps: DepsMut,
    router: Option<Contract>,
    enabled: Option<bool>,
    only_band: Option<bool>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    CONFIG.save(
        deps.storage,
        &Config {
            router: router.unwrap_or(config.router),
            enabled: enabled.unwrap_or(config.enabled),
            only_band: only_band.unwrap_or(config.only_band),
        },
    )?;

    Ok(
        Response::new()
        .set_data(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?)
    )
}

fn mod_basket(
    deps: DepsMut,
    mod_basket: Vec<(String, Uint128)>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let self_symbol = SYMBOL.load(deps.storage)?;

    let basket = BASKET.load(deps.storage)?;
    let symbols: Vec<String> = basket.clone().into_iter().map(|(sym, _, _)| sym).collect();
    let mut prices = fetch_prices(deps.as_ref(), &config, symbols.clone())?;
    // target previous price
    let target = eval_index(prices.clone(), basket.clone());

    let mut weights: Vec<(String, Uint128)> =
        basket.into_iter().map(|(sym, w, _)| (sym, w)).collect();
    let mut new_symbols = vec![];

    // Update weights
    for (mod_sym, mod_weight) in mod_basket.into_iter() {
        // Disallow recursive symbols
        if mod_sym == self_symbol {
            return Err(StdError::generic_err(format!(
                "Recursive symbol {}",
                self_symbol
            )));
        }

        // gather new symbols for fetching
        if !symbols.contains(&mod_sym) {
            new_symbols.push(mod_sym.clone());
        }

        // remove previous weight
        if let Some(i) = weights.iter().position(|(sym, _)| *sym == mod_sym) {
            weights.remove(i);
        } else if mod_weight.is_zero() {
            return Err(StdError::generic_err(format!(
                "Cannot remove symbol that does not exist {}",
                mod_sym
            )));
        }

        // add new/updated weights
        if !mod_weight.is_zero() {
            weights.push((mod_sym, mod_weight));
        }
    }

    // Verify new weights
    if weights.is_empty() {
        return Err(StdError::generic_err("Basket cannot be empty"));
    }
    if weights
        .clone()
        .into_iter()
        .map(|(_, w)| w.u128())
        .sum::<u128>()
        != 10u128.pow(18)
    {
        return Err(StdError::generic_err("Weights must add to 100%"));
    }

    prices.extend(fetch_prices(deps.as_ref(), &config, new_symbols)?);

    let constants = build_constants(weights.clone(), prices.clone(), target.rate);

    // Recalculate the constants
    let new_basket: Vec<(String, Uint128, Uint128)> = weights
        .into_iter()
        .map(|(sym, w)| (sym.clone(), w, constants[&sym]))
        .collect();
    BASKET.save(deps.storage, &new_basket)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateConfig {
        status: ResponseStatus::Success,
    })?))
}

fn try_query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

fn try_query_price(
    deps: Deps,
    key: String,
) -> StdResult<OraclePrice> {
    if key != SYMBOL.load(deps.storage)? {
        return Err(StdError::generic_err(format!(
            "Missing price feed for {}",
            key
        )));
    }

    let config = CONFIG.load(deps.storage)?;

    let basket = BASKET.load(deps.storage)?;
    let symbols: Vec<String> = basket.clone().into_iter().map(|(sym, _, _)| sym).collect();
    let prices = fetch_prices(deps, &config, symbols)?;
    let index = eval_index(prices, basket);

    Ok(OraclePrice::new(key, index))
}
