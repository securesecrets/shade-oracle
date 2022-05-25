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
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_cosmwasm_math_compat::{self, Uint512};
use secret_toolkit::utils::{pad_query_result, pad_handle_result};
use std::collections::HashMap;

use shade_oracles::{
    common::{
        OraclePrice, Contract,
        ResponseStatus, BLOCK_SIZE,
        querier::query_prices,
    },
    band::ReferenceData,
    storage::{Item},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Asset {
    symbol: String,
    weight: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admins: Vec<HumanAddr>,
    pub router: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    admins: Option<Vec<HumanAddr>>,
    router: Contract,
    symbol: String,
    basket: HashMap<String, Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    ModBasket {
        basket: HashMap<String, Uint128>,
    },
    UpdateConfig {
        admins: Option<Vec<HumanAddr>>,
        router: Option<Contract>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    ModBasket {
        status: ResponseStatus,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    GetPrice {
        symbol: String,
    },
    GetPrices {
        symbols: Vec<String>,
    },
    GetConfig { },
}

/*
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    GetPrice {
        price: Uint128,
    },
    UpdateConfig {
        config: Config,
    },
}
*/

const CONFIG: Item<Config> = Item::new("config");
const SYMBOL: Item<String> = Item::new("symbol");
const BASKET: Item<HashMap<String, Uint128>> = Item::new("basket");

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

    if msg.basket.contains_key(&msg.symbol) {
        return Err(StdError::generic_err(format!("Recursive symbol {}", msg.symbol)));
    }

    CONFIG.save(&mut deps.storage, &config)?;
    BASKET.save(&mut deps.storage, &msg.basket)?;
    SYMBOL.save(&mut deps.storage, &msg.symbol)?;

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

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> StdResult<Binary> {
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
            }
        }, BLOCK_SIZE)
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
    basket: HashMap<String, Uint128>,
) -> StdResult<HandleResponse> {

    let config = CONFIG.load(&deps.storage)?;

    if !config.admins.contains(&env.message.sender) {
        return Err(StdError::unauthorized());
    }

    let mut cur_basket = BASKET.load(&deps.storage)?;
    let self_symbol = SYMBOL.load(&deps.storage)?;

    for (mod_sym, mod_weight) in basket.iter() {
        // Disallow adding recursive symbol e.g. SILK basket containing SILK
        if *mod_sym == self_symbol {
            return Err(StdError::generic_err(format!("Recursive symbol {}", self_symbol)));
        }

        // Remove 0 weights
        if mod_weight.is_zero() {
            cur_basket.remove(mod_sym);
        }
        // Add/Update others
        else {
            cur_basket.insert(mod_sym.clone(), *mod_weight);
        }
    }

    BASKET.save(&mut deps.storage, &cur_basket)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn eval_basket<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    basket: HashMap<String, Uint128>,
) -> StdResult<Uint128> {

    let config = CONFIG.load(&deps.storage)?;

    let symbols = basket.keys().cloned().collect();

    let mut weight_sum = Uint512::zero();
    let mut index_price = Uint512::zero();

    for price in query_prices(&config.router, &deps.querier, symbols)? {
        index_price += Uint512::from(price.price.rate.u128()) * Uint512::from(basket.get(&price.symbol).unwrap().u128())
                / Uint512::from(10u128.pow(18));
    }

    Ok(Uint128(
        secret_cosmwasm_math_compat::Uint128::try_from(
            index_price
                .checked_mul(Uint512::from(10u128.pow(18)))?
                .checked_div(weight_sum)?,
        )?
        .u128(),
    ))
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

    to_binary(&OraclePrice::new( symbol, 
        ReferenceData {
            rate: eval_basket(&deps, BASKET.load(&deps.storage)?)?,
            last_updated_base: 0,
            last_updated_quote: 0,
        }))
}
