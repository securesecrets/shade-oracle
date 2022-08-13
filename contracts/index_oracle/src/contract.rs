use cosmwasm_std::{
    entry_point, Decimal256, DepsMut, MessageInfo, QueryResponse, Uint128,
};
use cosmwasm_std::{to_binary, Deps, Env, Response, StdError, StdResult};
use shade_oracles::common::Oracle;
use shade_oracles::core::better_secret_math::core::{muldiv_fp};
use shade_oracles::interfaces::index_oracle::{
    BasketResponse, BasketResponseItem, BasketSymbols, BtrBasket,
    Symbol, Target, TargetResponse,
};
use std::vec;
use std::{cmp::min, collections::HashMap};

use shade_oracles::{
    core::{
        better_secret_math::{core::muldiv, U256},
        pad_handle_result, pad_query_result, ResponseStatus,
    },
    interfaces::{
        band::ReferenceData,
        common::{
            querier::{query_band_prices, query_prices},
            CommonConfig, OraclePrice,
        },
        index_oracle::{ExecuteMsg, HandleAnswer, InstantiateMsg, QueryMsg},
    },
    storage::{GenericItemStorage, GenericMapStorage, ItemStorage},
    BLOCK_SIZE,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = IndexOracle.init_config(deps.storage, deps.api, msg.config)?;

    if msg.basket.is_empty() {
        return Err(StdError::generic_err("Basket cannot be empty"));
    }

    let mut symbols: Vec<String> = vec![];
    let mut weight_sum = Decimal256::zero();

    for (sym, weight) in &msg.basket {
        weight_sum += weight;
        symbols.push(sym.clone());
    }

    let sym_slice = symbols.as_slice();

    for symbol in sym_slice {
        if symbol.eq(&msg.symbol) {
            return Err(StdError::generic_err(format!(
                "Recursive symbol {}",
                msg.symbol
            )));
        }
    }

    if weight_sum != Decimal256::percent(100) {
        return Err(StdError::generic_err(format!(
            "Weights must add to 100%, {}",
            weight_sum
        )));
    }

    let prices = fetch_prices(deps.as_ref(), &config, symbols.as_slice())?;
    let constants = build_constants(msg.basket.as_slice(), prices, msg.target.into())?;

    for (sym, w) in msg.basket {
        let w: U256 = w.into();
        BtrBasket::save(deps.storage, sym.as_str(), &(w, constants[&sym]))?;
    }

    Target(msg.target).save(deps.storage)?;
    Symbol(msg.symbol).save(deps.storage)?;
    BasketSymbols::save(deps.storage, &symbols)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let mut config = IndexOracle.verify_admin(deps.storage, &deps.querier, info)?;
    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateConfig { updates } => {
                IndexOracle.try_update_config(deps, updates, &mut config)
            }
            ExecuteMsg::ModBasket { basket, .. } => mod_basket(deps, basket),
            ExecuteMsg::UpdateTarget { new_target } => {
                if let Some(new_target) = new_target {
                    Target(new_target).save(deps.storage)?;
                }
                Ok(
                    Response::new().set_data(to_binary(&HandleAnswer::UpdateTarget {
                        status: ResponseStatus::Success,
                    })?),
                )
            }
        },
        BLOCK_SIZE,
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => {
                to_binary(&IndexOracle.config_resp(CommonConfig::load(deps.storage)?))
            }
            /* add 'symbol' so we can error if its the wrong oracle
             * Prevents router failure from causing economic failure
             */
            QueryMsg::GetPrice { key, .. } => {
                let config = IndexOracle.can_query_price(deps, &key)?;
                to_binary(
                    &IndexOracle.price_resp(IndexOracle.try_query_price(deps, &env, key, &config)?),
                )
            }
            QueryMsg::GetPrices { keys } => {
                let config = IndexOracle.can_query_prices(deps, keys.as_slice())?;
                to_binary(
                    &IndexOracle
                        .prices_resp(IndexOracle.try_query_prices(deps, &env, keys, &config)?),
                )
            }
            QueryMsg::Basket {} => to_binary(&BasketResponse {
                basket: query_basket(deps)?,
            }),
            QueryMsg::GetTarget {} => to_binary(&TargetResponse {
                target: Target::load(deps.storage)?.0,
            }),
        },
        BLOCK_SIZE,
    )
}

fn query_basket(deps: Deps) -> StdResult<Vec<BasketResponseItem>> {
    let symbols = BasketSymbols::load(deps.storage)?;
    let mut basket_response_items = vec![];
    for symbol in symbols {
        let item = BtrBasket::load(deps.storage, symbol.as_str())?;
        let item: BasketResponseItem = (symbol, item.0.into(), item.1.as_u128().into());
        basket_response_items.push(item)
    }
    Ok(basket_response_items)
}

fn build_constants(
    weights: &[(String, Decimal256)],
    prices: HashMap<String, ReferenceData>,
    target: U256,
) -> StdResult<HashMap<String, U256>> {
    let mut constants: HashMap<String, U256> = HashMap::new();
    for (sym, weight) in weights {
        constants.insert(
            sym.clone(),
            muldiv((*weight).into(), target, prices[sym].rate.into())?,
        );
    }
    Ok(constants)
}

fn eval_index(
    prices: HashMap<String, ReferenceData>,
    basket: &[(String, U256, U256)],
) -> StdResult<ReferenceData> {
    let mut index_price = U256::ZERO;
    let mut last_updated_base = 0u64;
    let mut last_updated_quote = 0u64;

    for (sym, _, constant) in basket {
        index_price += muldiv_fp(U256::new(prices[sym].rate.u128()), *constant)?;
        last_updated_base = min(last_updated_base, prices[sym].last_updated_base);
        last_updated_quote = min(last_updated_quote, prices[sym].last_updated_quote);
    }
    Ok(ReferenceData {
        rate: Uint128::from(index_price.as_u128()),
        last_updated_base,
        last_updated_quote,
    })
}

fn fetch_prices<'a>(
    deps: Deps,
    config: &CommonConfig,
    symbols: impl IntoIterator<Item = &'a String>,
) -> StdResult<HashMap<String, ReferenceData>> {
    let mut price_data = HashMap::new();
    let symbols = symbols
        .into_iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>();
    let symbols_slice = symbols.as_slice();
    let prices_resp = if config.only_band {
        query_band_prices(&config.router, &deps.querier, symbols_slice)
    } else {
        query_prices(&config.router, &deps.querier, symbols_slice)
    };
    match prices_resp {
        Ok(prices) => {
            for oracle_price in prices {
                price_data.insert(oracle_price.key().clone(), oracle_price.data().clone());
            }
        }
        Err(e) => {
            return Err(StdError::generic_err(format!(
                "Failed to query {} from router {}, '{}'",
                symbols
                    .into_iter()
                    .map(|sym| sym + ",")
                    .collect::<String>(),
                config.router.address.as_str(),
                e
            )))
        }
    }
    Ok(price_data)
}

fn mod_basket(
    deps: DepsMut,
    mod_basket: impl IntoIterator<Item = (String, Decimal256)>,
) -> StdResult<Response> {
    let config = CommonConfig::load(deps.storage)?;

    let self_symbol = Symbol::load(deps.storage)?.0;
    let symbols = BasketSymbols::load(deps.storage)?;

    let basket = BtrBasket::load_basket(deps.storage, &symbols)?;
    let mut prices = fetch_prices(deps.as_ref(), &config, &symbols)?;
    // target previous price
    let target = eval_index(prices.clone(), basket.as_slice())?;

    let mut weights: Vec<(String, Decimal256)> = basket
        .into_iter()
        .map(|(sym, w, _)| (sym, w.into()))
        .collect();
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
        // if all of the symbols don't match mod_sym then mod_sym is new
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
        .map(|(_, w)| w)
        .sum::<Decimal256>()
        != Decimal256::percent(100)
    {
        return Err(StdError::generic_err("Weights must add to 100%"));
    }

    prices.extend(fetch_prices(
        deps.as_ref(),
        &config,
        new_symbols.as_slice(),
    )?);

    let constants = build_constants(weights.as_slice(), prices.clone(), target.rate.into())?;

    let mut new_symbols = vec![];
    // Recalculate the constants
    for (sym, w) in weights {
        let w: U256 = w.into();
        BtrBasket::save(deps.storage, sym.as_str(), &(w, constants[&sym]))?;
        new_symbols.push(sym);
    }

    BasketSymbols::save(deps.storage, &new_symbols)?;

    Ok(
        Response::new().set_data(to_binary(&HandleAnswer::ModBasket {
            status: ResponseStatus::Success,
        })?),
    )
}

pub struct IndexOracle;

impl Oracle for IndexOracle {
    fn try_query_price(
        &self,
        deps: Deps,
        _env: &Env,
        key: String,
        config: &shade_oracles::common::CommonConfig,
    ) -> StdResult<OraclePrice> {
        if key != Symbol::load(deps.storage)?.0 {
            return Err(StdError::generic_err(format!(
                "Missing price feed for {}",
                key
            )));
        }

        let symbols = BasketSymbols::load(deps.storage)?;
        let basket = BtrBasket::load_basket(deps.storage, &symbols)?;
        let prices = fetch_prices(deps, config, symbols.iter())?;
        let index = eval_index(prices, &basket)?;

        Ok(OraclePrice::new(key, index))
    }
}
