use cosmwasm_std::{attr, entry_point, Binary, Decimal256, DepsMut, MessageInfo, Uint128, Uint64};
use cosmwasm_std::{to_binary, Deps, Env, Response};
use shade_oracles::common::querier::validate_permission;
use shade_oracles::common::{PriceResponse, PricesResponse, ShadeOraclePermissions};
use shade_oracles::core::{Contract, RawContract};
use shade_oracles::create_attr_action;
use shade_oracles::interfaces::index::{error::*, msg::*, *};
use shade_oracles::{
    common::status::GlobalStatus,
    core::{pad_handle_result, pad_query_result},
    interfaces::{
        band::ReferenceData,
        common::{querier::query_band_prices, OraclePrice},
        router::querier::get_admin_auth,
    },
    ssp::ItemStorage,
    BLOCK_SIZE,
};
use std::vec;

create_attr_action!("_index_oracle");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> IndexOracleResult<Response> {
    let router = msg.router.into_valid(deps.api)?;
    IndexOracle::init_status(deps.storage)?;
    let mut index_oracle = IndexOracle::init(
        msg.symbol,
        router.clone(),
        msg.when_stale,
        msg.basket,
        msg.target,
        &env.block.time,
    )?;
    let prices = query_band_prices(
        &router,
        &deps.querier,
        index_oracle.asset_symbols.as_slice(),
    )?;
    index_oracle.compute_fixed_weights(&prices)?;
    index_oracle.save(deps.storage)?;

    Ok(Response::new().add_attributes(vec![attr_action!("instantiate")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> IndexOracleResult<Response> {
    let index_oracle = IndexOracle::load(deps.storage)?;
    let resp = match msg {
        ExecuteMsg::ComputeIndex {} => try_compute_index(deps, env, info, index_oracle),
        ExecuteMsg::Admin(msg) => try_admin_msg(deps, env, info, msg, index_oracle),
    }?;
    Ok(pad_handle_result(Ok(resp), BLOCK_SIZE)?)
}

pub fn try_compute_index(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    mut oracle: IndexOracle,
) -> IndexOracleResult<Response> {
    IndexOracle::require_can_run(deps.storage, true, false, false)?;
    let router = oracle.config.router.clone();
    let symbols = oracle.asset_symbols.clone();
    let prices = fetch_prices(deps.as_ref(), &router, &symbols)?;
    oracle.compute_target(prices.as_ref(), &env.block.time)?;
    oracle.save(deps.storage)?;
    Ok(Response::new().add_attributes(vec![
        attr_action!("compute_index"),
        attr("new_target", oracle.target.value.to_string()),
        attr("is_frozen", oracle.target.frozen.to_string()),
    ]))
}

pub fn try_unfreeze(
    deps: DepsMut,
    env: Env,
    mut oracle: IndexOracle,
) -> IndexOracleResult<Response> {
    let prices = query_band_prices(
        &oracle.config.router,
        &deps.querier,
        oracle.asset_symbols.as_slice(),
    )?;
    oracle.rollback(prices.as_slice(), &env.block.time)?;
    oracle.save(deps.storage)?;
    Ok(Response::new().add_attributes(vec![
        attr_action!("unfreeze"),
        attr("new_target", oracle.target.value.to_string()),
    ]))
}

pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    mut oracle: IndexOracle,
    symbol: Option<String>,
    router: Option<RawContract>,
    when_stale: Option<Uint64>,
) -> IndexOracleResult<Response> {
    oracle.config.router = match router {
        Some(router) => router.into_valid(deps.api)?,
        None => oracle.config.router,
    };
    oracle.config.symbol = symbol.unwrap_or(oracle.config.symbol);
    oracle.config.when_stale = match when_stale {
        Some(when_stale) => when_stale.u64(),
        None => oracle.config.when_stale,
    };
    oracle.config.save(deps.storage)?;

    Ok(Response::new().add_attributes(vec![attr_action!("update_config")]))
}

pub fn try_update_target(
    deps: DepsMut,
    env: Env,
    mut oracle: IndexOracle,
    new_target: Uint128,
) -> IndexOracleResult<Response> {
    let prices = query_band_prices(
        &oracle.config.router,
        &deps.querier,
        oracle.asset_symbols.as_slice(),
    )?;
    oracle.target.value = new_target.into();
    oracle.target.last_updated = env.block.time.seconds();
    oracle.compute_fixed_weights(prices.as_slice())?;
    oracle.save(deps.storage)?;
    Ok(Response::new().add_attributes(vec![
        attr_action!("unfreeze"),
        attr("new_target", new_target),
    ]))
}

pub fn try_admin_msg(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AdminMsg,
    oracle: IndexOracle,
) -> IndexOracleResult<Response> {
    let router = oracle.config.router.clone();
    let admin_auth = get_admin_auth(&router, &deps.querier)?.config.admin_auth;
    let no_perms = validate_permission(
        &deps.querier,
        ShadeOraclePermissions::SuperAdmin,
        &info.sender,
        &admin_auth,
    )
    .is_err()
        && validate_permission(
            &deps.querier,
            ShadeOraclePermissions::SilkAssembly,
            &info.sender,
            &admin_auth,
        )
        .is_err();
    if no_perms {
        Err(IndexOracleError::Unauthorized { user: info.sender })
    } else {
        match msg {
            AdminMsg::UpdateStatus { status } => {
                IndexOracle::update_status(deps.storage, status)?;
                Ok(Response::new().add_attributes(vec![attr_action!("update_status")]))
            }
            _ => {
                IndexOracle::require_can_run(deps.storage, true, true, false)?;
                match msg {
                    AdminMsg::ModBasket { basket } => try_mod_basket(deps, env, basket, oracle),
                    AdminMsg::UpdateConfig {
                        symbol,
                        router,
                        when_stale,
                    } => try_update_config(deps, env, oracle, symbol, router, when_stale),
                    AdminMsg::UpdateTarget { new_target } => {
                        try_update_target(deps, env, oracle, new_target)
                    }
                    AdminMsg::Unfreeze {} => try_unfreeze(deps, env, oracle),
                    _ => panic!("code should never come here"),
                }
            }
        }
    }
}

/// Used in cases where we want to tolerate Band being down.
pub fn fetch_prices<'a>(
    deps: Deps,
    router: &Contract,
    symbols: impl IntoIterator<Item = &'a String>,
) -> IndexOracleResult<Option<Vec<OraclePrice>>> {
    let symbols = symbols
        .into_iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>();
    let symbols_slice = symbols.as_slice();
    match query_band_prices(router, &deps.querier, symbols_slice) {
        Ok(prices) => Ok(Some(prices)),
        Err(_) => Ok(None),
    }
}

pub fn try_mod_basket(
    deps: DepsMut,
    env: Env,
    mod_basket: impl IntoIterator<Item = (String, Decimal256)>,
    mut oracle: IndexOracle,
) -> IndexOracleResult<Response> {
    // Compute target with old weights
    let router = oracle.config.router.clone();
    let prices = query_band_prices(&router, &deps.querier, oracle.asset_symbols.as_slice())?;
    oracle.compute_target(Some(&prices), &env.block.time)?;
    // Update weights
    oracle.update_basket(mod_basket)?;

    let new_prices = query_band_prices(&router, &deps.querier, &oracle.asset_symbols)?;

    let new_prices = new_prices.as_slice();
    oracle.compute_fixed_weights(new_prices)?;
    oracle.save(deps.storage)?;

    Ok(Response::new().add_attributes(vec![attr_action!("mod_basket")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> IndexOracleResult<Binary> {
    let mut oracle = IndexOracle::load(deps.storage)?;
    let now = env.block.time.seconds();

    let binary = match msg {
        QueryMsg::GetPrice { key } => {
            IndexOracle::require_can_run(deps.storage, true, false, false)?;
            if key != oracle.config.symbol {
                return Err(IndexOracleError::UnsupportedSymbol { symbol: key });
            }
            let prices =
                fetch_prices(deps, &oracle.config.router, oracle.asset_symbols.as_slice())?;
            oracle.compute_target(prices.as_ref(), &env.block.time)?;
            let price = OraclePrice::new(
                oracle.config.symbol,
                ReferenceData {
                    rate: oracle.target.value.into(),
                    last_updated_base: now,
                    last_updated_quote: now,
                },
            );
            to_binary(&PriceResponse { price })
        }
        QueryMsg::GetPrices { keys } => {
            IndexOracle::require_can_run(deps.storage, true, false, false)?;
            for key in &keys {
                if key.eq(&oracle.config.symbol) {
                    return Err(IndexOracleError::UnsupportedSymbol {
                        symbol: key.clone(),
                    });
                }
            }
            let prices =
                fetch_prices(deps, &oracle.config.router, oracle.asset_symbols.as_slice())?;
            oracle.compute_target(prices.as_ref(), &env.block.time)?;
            let price = OraclePrice::new(
                oracle.config.symbol,
                ReferenceData {
                    rate: oracle.target.value.into(),
                    last_updated_base: now,
                    last_updated_quote: now,
                },
            );
            let prices = vec![price; keys.capacity()];
            to_binary(&PricesResponse { prices })
        }
        QueryMsg::GetIndexData {} => {
            IndexOracle::require_can_run(deps.storage, true, true, false)?;
            let prices =
                fetch_prices(deps, &oracle.config.router, oracle.asset_symbols.as_slice())?;
            oracle.compute_target(prices.as_ref(), &env.block.time)?;
            let basket = oracle
                .basket
                .iter()
                .map(|(k, v)| IndexAsset {
                    symbol: k.into(),
                    weight: v.clone().into(),
                })
                .collect::<Vec<IndexAsset>>();
            to_binary(&IndexDataResponse {
                symbol: oracle.config.symbol,
                router: oracle.config.router,
                when_stale: Uint64::new(oracle.config.when_stale),
                target: oracle.target.into(),
                basket,
            })
        }
        QueryMsg::GetBasket {} => {
            IndexOracle::require_can_run(deps.storage, true, true, false)?;
            let basket = oracle
                .basket
                .iter()
                .map(|(k, v)| (k.clone(), v.initial.into(), v.fixed.into()))
                .collect::<Vec<(String, Decimal256, Decimal256)>>();
            to_binary(&BasketResponse { basket })
        }
    }?;
    Ok(pad_query_result(Ok(binary), BLOCK_SIZE)?)
}
