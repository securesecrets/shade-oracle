use cosmwasm_std::{
    to_binary,
    Api,
    Binary,
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
    Order,
    QueryResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
    storage::{Item, Map},
    index_oracle::{
        InitMsg, HandleMsg, HandleAnswer, QueryMsg, QueryAnswer,
    },
    router,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admins: Vec<HumanAddr>,
    pub router: Contract,
}

const CONFIG: Item<Config> = Item::new("config");
const SYMBOL: Item<String> = Item::new("symbol");
// TODO: Change to a single Map<sym, (weight, constant)>
const WEIGHTS: Map<String, Uint128> = Map::new("weights");
const CONSTANTS: Map<String, Uint128> = Map::new("constants");

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

    let (symbols, _): (Vec<String>, Vec<Uint128>) = msg.basket.clone().into_iter().unzip();

    if symbols.contains(&msg.symbol) {
        return Err(StdError::generic_err(format!("Recursive symbol {}", msg.symbol)));
    }

    let basket: HashMap<_,_> = msg.basket.clone().into_iter().collect();


    let mut weight_sum = Uint128::zero();
    for (sym, weight) in basket.clone() {
        weight_sum += weight;
        WEIGHTS.save(&mut deps.storage, sym, &weight)?;
    }

    if weight_sum != Uint128(10u128.pow(18)) {
        return Err(StdError::generic_err(format!("Weights must add to 100%, {}", weight_sum)));
    }

    for (sym, c) in build_constants(basket, fetch_prices(deps, symbols)?, msg.target) {
        CONSTANTS.save(&mut deps.storage, sym, &c)?;
    }

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
            QueryMsg::GetPrice { symbol, .. } => to_binary(&try_query_price(deps, symbol)?),
            QueryMsg::GetPrices { symbols } => {
                let mut prices = vec![];
                for symbol in symbols {
                    prices.push(try_query_price(deps, symbol)?);
                }
                to_binary(&prices)
            },
            QueryMsg::Basket { } => {
                let weights: HashMap<String,Uint128> = WEIGHTS
                    .range(&deps.storage, 
                           None, None, 
                           Order::Ascending
                    ).map(|i| i.ok().unwrap()).collect();
                let constants: HashMap<String,Uint128> = CONSTANTS 
                    .range(&deps.storage, 
                           None, None, 
                           Order::Ascending
                    ).map(|i| i.ok().unwrap()).collect();

                let mut basket = vec![];
                for (sym, w) in weights {
                    basket.push((sym.clone(), w, constants[&sym]));
                }
                to_binary(&QueryAnswer::Basket {
                    basket,

                })
            },
        }, BLOCK_SIZE)
}

fn build_constants(
    basket: HashMap<String, Uint128>,
    prices: HashMap<String, Uint128>,
    target: Uint128,
) -> HashMap<String, Uint128> {

    let mut constants: HashMap<String, Uint128> = HashMap::new();

    for (sym, weight) in basket {
        constants.insert(sym.clone(), weight.multiply_ratio(target, prices[&sym]));
    }

    constants
}

fn eval_index(
    prices: HashMap<String, Uint128>,
    constants: HashMap<String, Uint128>,
) -> Uint128 {

    //assert_eq!(constants.keys().cloned().collect::<Vec<_>>(), prices.keys().cloned().collect::<Vec<_>>());

    let symbols: Vec<String> = constants.keys().cloned().collect();
    let mut index_price = Uint512::zero();

    for sym in symbols {
        index_price += Uint512::from(prices[&sym].u128()) * Uint512::from(constants[&sym].u128()) / Uint512::from(10u128.pow(18));
    }
    Uint128(compat::Uint128::try_from(index_price).ok().unwrap().u128())
}

fn fetch_prices<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbols: Vec<String>,
) -> StdResult<HashMap<String, Uint128>> {

    let config = CONFIG.load(&deps.storage)?;

    let oracles: Vec<router::OracleResponse> = router::QueryMsg::GetOracles { keys: symbols }.query(
        &deps.querier,
        config.router.code_hash,
        config.router.address,
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

    let mut weights: HashMap<_,_> = WEIGHTS.range(&deps.storage, None, None, Order::Ascending).map(|i| i.ok().unwrap()).collect();
    let mut symbols: Vec<String> = weights.keys().cloned().collect();

    // Update weights
    for (mod_sym, mod_weight) in mod_basket.into_iter() {

        // Disallow recursive symbols
        if mod_sym == self_symbol {
            return Err(StdError::generic_err(format!("Recursive symbol {}", self_symbol)));
        }

        // Remove 0 weights
        if mod_weight.is_zero() {
            weights.remove(&mod_sym.to_string());
            WEIGHTS.remove(&mut deps.storage, mod_sym);
        }
        // Add/Update others
        else {
            weights.insert(mod_sym.clone(), mod_weight);
            WEIGHTS.save(&mut deps.storage, mod_sym.clone(), &mod_weight)?;

            // Add new symbols for price querying
            if !symbols.contains(&mod_sym) {
                symbols.push(mod_sym.clone());
            }
        }
    }

    // Verify new weights
    if weights.is_empty() {
        return Err(StdError::generic_err("Basket cannot be empty"));
    }
    if weights.clone().into_iter().map(|(_, w)| w.u128()).sum::<u128>() != 10u128.pow(18) {
        return Err(StdError::generic_err("Weights must add to 100%"));
    }

    let prices = fetch_prices(deps, symbols)?;

    // get target price to calibrate new constants
    let constants: HashMap<_,_> = CONSTANTS.range(&deps.storage, None, None, Order::Ascending).map(|i| i.ok().unwrap()).collect();
    let target = eval_index(prices.clone(), constants.clone());

    let new_const = build_constants(weights.clone(), prices.clone(), target);

    // Recalculate the constants
    for (sym, c) in new_const.clone() {
        CONSTANTS.save(&mut deps.storage, sym, &c)?;
    }
    for (sym, _) in constants {
        if !new_const.contains_key(&sym.clone()) {
            CONSTANTS.remove(&mut deps.storage, sym);
        }
    }

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
    Ok(CONFIG.load(&deps.storage)?)
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
) -> StdResult<Binary> {

    if symbol != SYMBOL.load(&deps.storage)? {
        return Err(StdError::generic_err(format!("Missing price feed for {}", symbol)));
    }

    let constants: HashMap<_,_> = CONSTANTS.range(&deps.storage, None, None, Order::Ascending).map(|i| i.ok().unwrap()).collect();
    let (symbols, _): (Vec<String>, Vec<Uint128>) = constants.clone().into_iter().unzip();
    let prices = fetch_prices(deps, symbols.clone())?;

    to_binary(
        &OraclePrice::new(
            symbol,
            ReferenceData {
                rate: eval_index(prices, constants),
                //TODO these should be the minimum found
                last_updated_base: 0,
                last_updated_quote: 0,
            }
        )
    )
}
