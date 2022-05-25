/*
use mulberry_utils::{
    auth::{assert_admin, load_admin, save_admin},
    common::types::{CanonicalContract, Contract, ResponseStatus},
    scrt::{
        to_binary, Api, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
        QueryResult, StdError, StdResult, Storage, BLOCK_SIZE,
    },
    secret_toolkit::utils::{pad_handle_result, pad_query_result, Query},
    storage::bincode_config::{load, save},
};
*/
use cosmwasm_std::{
    debug_print,
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
use shade_protocol::{
    contract_interfaces::oracles::band::{
        self, BandQuery
    },
    utils::{
        asset::Contract,
        storage::default::{SingletonStorage},
        generic_response::ResponseStatus,
    },
};

use shade_oracles::band::ReferenceData;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Asset {
    symbol: String,
    weight: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Basket(pub Vec<Asset>);

impl SingletonStorage for Basket {
    const NAMESPACE: &'static [u8] = b"basket";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Symbol(pub String);

impl SingletonStorage for Symbol {
    const NAMESPACE: &'static [u8] = b"symbol";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admins: Vec<HumanAddr>,
    pub band: Contract,
}

impl SingletonStorage for Config {
    const NAMESPACE: &'static [u8] = b"config";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    admins: Option<Vec<HumanAddr>>,
    band: Contract,
    symbol: Symbol,
    basket: Basket,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    ModBasket {
        basket: Basket,
    },
    UpdateConfig {
        admins: Option<Vec<HumanAddr>>,
        band: Option<Contract>,
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

/* TODO
 * Router could accept a list of symbols, return a list of routes
 * This would allow the end oracle to combine queries to the same oracle
 */

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
        band: msg.band,
    };

    if msg.basket.0.iter().find(|asset| asset.symbol == msg.symbol.0).is_some() {
        return Err(StdError::generic_err(format!("Recursive symbol {}", msg.symbol.0)));
    }

    config.save(&mut deps.storage)?;
    msg.basket.save(&mut deps.storage)?;
    msg.symbol.save(&mut deps.storage)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateConfig {
            admins,
            band,
        } => try_update_config(deps, env, admins, band),
        HandleMsg::ModBasket { basket, .. } => mod_basket(deps, env, basket),
    }
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admins: Option<Vec<HumanAddr>>,
    band: Option<Contract>,
) -> StdResult<HandleResponse> {
    let mut config = Config::load(&deps.storage)?;

    if !config.admins.contains(&env.message.sender) {
        return Err(StdError::unauthorized());
    }

    if let Some(admins) = admins {
        config.admins = admins;
    }

    if let Some(band) = band {
        config.band = band;
    }

    config.save(&mut deps.storage)?;

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
    basket: Basket,
) -> StdResult<HandleResponse> {

    let config = Config::load(&deps.storage)?;

    if !config.admins.contains(&env.message.sender) {
        return Err(StdError::unauthorized());
    }

    let mut cur_basket = Basket::load(&deps.storage)?;
    let cur_symbols: Vec<String> = cur_basket.0.iter().map(|a| a.symbol).collect();

    let self_symbol = Symbol::load(&deps.storage)?;

    if basket.0.iter().find(|asset| asset.symbol == self_symbol.0).is_some() {
        return Err(StdError::generic_err(format!("Recursive symbol {}", self_symbol.0)));
    }

    // TODO Disallow adding the symbol for this basket

    for mod_asset in basket.0.iter() {
        match cur_basket.0.iter().position(|a| a.symbol == mod_asset.symbol) {
            Some(i) => {
                if mod_asset.weight > Uint128::zero() {
                    // Update
                    cur_basket.0[i].weight = mod_asset.weight;
                }
                else {
                    // Remove
                    cur_basket.0.remove(i);
                }
            },
            None => {
                // Add new
                cur_basket.0.push(mod_asset.clone());
            }
        }
    }

    cur_basket.0.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    cur_basket.save(&mut deps.storage)?;

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
    basket: Basket,
) -> StdResult<Uint128> {

    let config = Config::load(&deps.storage)?;

    let mut weight_sum = Uint512::zero();
    let mut price = Uint512::zero();

    let mut band_bases = vec![];
    let mut band_quotes = vec![];
    let mut band_weights = vec![];

    for asset in basket.0 {
        weight_sum += Uint512::from(asset.weight.u128());

        // query oracle

        if false { //TODO Check source
            // Non-band assets
            return Err(StdError::generic_err(format!(
                "EVAL INDEX DEX PAIRS {}",
                asset.symbol
            )));

        }
        else {
            // Combine band assets for bulk query
            band_weights.push(asset.weight);
            band_bases.push(asset.symbol.clone());
            band_quotes.push("USD".to_string());
        }
    }

    if band_bases.len() > 0 {
        //TODO get from shade implementation
        let ref_data = band::reference_data_bulk(
            deps,
            band_bases,
            band_quotes,
            config.band,
        )?;

        for (reference, weight) in ref_data.iter().zip(band_weights.iter()) {
            price += Uint512::from(reference.rate.u128()) * Uint512::from(weight.u128())
                / Uint512::from(10u128.pow(18));
        }
    }

    Ok(Uint128(
        secret_cosmwasm_math_compat::Uint128::try_from(
            price
                .checked_mul(Uint512::from(10u128.pow(18)))?
                .checked_div(weight_sum)?,
        )?
        .u128(),
    ))
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&try_query_config(deps)?),
        /* add 'symbol' so we can error if its the wrong oracle
         * Prevents router failure from causing economic failure
         */
        QueryMsg::GetPrice { symbol, .. } => to_binary(&try_query_price(deps, symbol)?),
        QueryMsg::GetPrices { symbols } => {
            let self_symbol = Symbol::load(&deps.storage)?;
            let ref_data = vec![];
            for symbol in symbols {
                if symbol != self_symbol.0 {
                    return Err(StdError::generic_err(format!("Missing price feed for {}", symbol)));
                }
                else {
                    ref_data.push(try_query_price(deps, symbol)?);
                }
            }
            to_binary(&ref_data)
        }
    }
    //pad_query_result(response, BLOCK_SIZE)
}

fn try_query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Config> {
    Ok(Config::load(&deps.storage)?)
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
) -> StdResult<ReferenceData> {

    if symbol != Symbol::load(&deps.storage)?.0 {
        return Err(StdError::generic_err(format!("Missing price feed for {}", symbol)));
    }

    let rate = eval_basket(&deps, Basket::load(&deps.storage)?)?;

    Ok(ReferenceData {
        rate,
        last_updated_base: 0,
        last_updated_quote: 0,
    })
}
