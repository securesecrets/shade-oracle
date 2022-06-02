use cosmwasm_std::{
    to_binary,
    Api,
    Env,
    Extern,
    HandleResponse,
    InitResponse,
    Querier,
    StdResult,
    StdError,
    Storage,
    HumanAddr,
    Uint128,
    QueryResult,
};
use secret_cosmwasm_math_compat::{self as compat, Uint512};
use secret_toolkit::utils::{Query, pad_query_result, pad_handle_result};

use std::collections::HashMap;

use shade_oracles::{
    common::{
        self,
        OraclePrice, Contract,
        ResponseStatus, BLOCK_SIZE,
        //querier::query_prices,
    },
    band::ReferenceData,
    storage::Item,
    index_oracle::{
        InitMsg, HandleMsg, HandleAnswer, QueryMsg, QueryAnswer,
        Config,
    },
    router,
};

const CONFIG: Item<Config> = Item::new("config");
const SYMBOL: Item<String> = Item::new("symbol");
// (symbol, weight, constant)
const BASKET: Item<Vec<(String, Uint128, Uint128)>> = Item::new("basket");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let config = Config {
        admins: match msg.admins {
            Some(mut a) => {
                if !a.contains(&env.message.sender) {
                    a.push(env.message.sender);
                }
                a
            }
            None => vec![env.message.sender],
        },
        router: msg.router,
    };

    CONFIG.save(&mut deps.storage, &config)?;
    SYMBOL.save(&mut deps.storage, &msg.symbol)?;

    if msg.basket.is_empty() {
        return Err(StdError::generic_err("Basket cannot be empty"));
    }

    let symbols: Vec<String> = msg.basket.clone().into_iter().map(|(sym, _)| sym).collect();

    if symbols.contains(&msg.symbol) {
        return Err(StdError::generic_err(format!("Recursive symbol {}", msg.symbol)));
    }

    let weight_sum: u128 = msg.basket.clone().into_iter().map(|(_, w)| w.u128()).sum();
    if weight_sum != 10u128.pow(18) {
        return Err(StdError::generic_err(format!("Weights must add to 100%, {}", weight_sum)));
    }

    let prices = fetch_prices(deps, &config, symbols)?;
    let constants = build_constants(msg.basket.clone(), prices, msg.target);

    let mut full_basket: Vec<(String, Uint128, Uint128)> = msg.basket.into_iter().map(|(sym, w)| (sym.clone(), w, constants[&sym])).collect();
    full_basket.sort();

    BASKET.save(&mut deps.storage, &full_basket)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    pad_handle_result(
        match msg {
            HandleMsg::UpdateConfig {
                admins,
                router,
            } => try_update_config(deps, env, admins, router),
            HandleMsg::ModBasket { basket, .. } => mod_basket(deps, env, basket),
        }, BLOCK_SIZE)
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
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
            },
            QueryMsg::Basket { } => {
                to_binary(&QueryAnswer::Basket {
                    basket: BASKET.load(&deps.storage)?,
                })
            },
        }, BLOCK_SIZE)
}

fn build_constants(
    weights: Vec<(String, Uint128)>,
    prices: HashMap<String, Uint128>,
    target: Uint128,
) -> HashMap<String, Uint128> {

    let mut constants: HashMap<String, Uint128> = HashMap::new();

    for (sym, weight) in weights {
        constants.insert(sym.clone(), weight.multiply_ratio(target, prices[&sym]));
    }
    constants
}

fn eval_index(
    prices: HashMap<String, Uint128>,
    basket: Vec<(String, Uint128, Uint128)>,
) -> Uint128 {

    let mut index_price = Uint512::zero();

    for (sym, _, constant) in basket {
        index_price += Uint512::from(prices[&sym].u128()) * Uint512::from(constant.u128()) / Uint512::from(10u128.pow(18));
    }
    Uint128(compat::Uint128::try_from(index_price).ok().unwrap().u128())
}

fn fetch_prices<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    config: &Config,
    symbols: Vec<String>,
) -> StdResult<HashMap<String, Uint128>> {

    let oracles: Vec<router::OracleResponse> = router::QueryMsg::GetOracles { keys: symbols }.query(
        &deps.querier,
        config.router.code_hash.clone(),
        config.router.address.clone(),
    )?;

    let mut oracle_data: HashMap<Contract, Vec<String>> = HashMap::new();
    for oracle in oracles {
        oracle_data.entry(oracle.oracle).or_insert(vec![]).push(oracle.key);
    }

    let mut price_data = HashMap::new();
    for (oracle, symbols) in oracle_data {
        let prices: Vec<OraclePrice> = common::QueryMsg::GetPrices { keys: symbols }.query(
            &deps.querier,
            oracle.code_hash,
            oracle.address,
        )?;

        for oracle_price in prices {
            price_data.insert(oracle_price.key.clone(), oracle_price.price.rate);
        }
    }

    Ok(price_data)
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admins: Option<Vec<HumanAddr>>,
    router: Option<Contract>,
) -> StdResult<HandleResponse> {

    let config = CONFIG.load(&deps.storage)?;

    if !config.admins.contains(&env.message.sender) {
        return Err(StdError::unauthorized());
    }

    CONFIG.save(&mut deps.storage, &Config {
        admins: match admins {
            Some(a) => a,
            None => config.admins
        },
        router: match router {
            Some(r) => r,
            None => config.router,
        },
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

fn mod_basket<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    mod_basket: Vec<(String, Uint128)>,
) -> StdResult<HandleResponse> {

    let config = CONFIG.load(&deps.storage)?;

    if !config.admins.contains(&env.message.sender) {
        return Err(StdError::unauthorized());
    }

    let self_symbol = SYMBOL.load(&deps.storage)?;

    let basket = BASKET.load(&deps.storage)?;
    let symbols: Vec<String> = basket.clone().into_iter().map(|(sym, _, _)| sym).collect();
    let mut prices = fetch_prices(deps, &config, symbols.clone())?;
    // target previous price
    let target = eval_index(prices.clone(), basket.clone());

    let mut weights: Vec<(String, Uint128)> = basket.into_iter().map(|(sym, w, _)| (sym, w)).collect();
    let mut new_symbols = vec![];

    // Update weights
    for (mod_sym, mod_weight) in mod_basket.into_iter() {

        // Disallow recursive symbols
        if mod_sym == self_symbol {
            return Err(StdError::generic_err(format!("Recursive symbol {}", self_symbol)));
        }

        // gather new symbols for fetching
        if !symbols.contains(&mod_sym) {
            new_symbols.push(mod_sym.clone());
        }

        // remove previous weight
        if let Some(i) = weights.iter().position(|(sym, _)| *sym == mod_sym) {
            weights.remove(i);
        }
        else if mod_weight.is_zero() {
            return Err(StdError::generic_err(format!("Cannot remove symbol that does not exist {}", mod_sym)));
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
    if weights.clone().into_iter().map(|(_, w)| w.u128()).sum::<u128>() != 10u128.pow(18) {
        return Err(StdError::generic_err("Weights must add to 100%"));
    }

    prices.extend(fetch_prices(deps, &config, new_symbols)?);

    let constants = build_constants(weights.clone(), prices.clone(), target);

    // Recalculate the constants
    let new_basket: Vec<(String, Uint128, Uint128)> = weights.into_iter().map(|(sym, w)| (sym.clone(), w, constants[&sym])).collect();
    BASKET.save(&mut deps.storage, &new_basket)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

fn try_query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Config> {
    CONFIG.load(&deps.storage)
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: String,
) -> StdResult<OraclePrice> {

    if key != SYMBOL.load(&deps.storage)? {
        return Err(StdError::generic_err(format!("Missing price feed for {}", key)));
    }

    let config = CONFIG.load(&deps.storage)?;

    let basket = BASKET.load(&deps.storage)?;
    let symbols: Vec<String> = basket.clone().into_iter().map(|(sym, _, _)| sym).collect();
    let prices = fetch_prices(deps, &config, symbols)?;

    Ok(OraclePrice::new(
        key,
        ReferenceData {
            rate: eval_index(prices, basket),
            //TODO these should be the minimum found
            last_updated_base: 0,
            last_updated_quote: 0,
        }
    ))
}
