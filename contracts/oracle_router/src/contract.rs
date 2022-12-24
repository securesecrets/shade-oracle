use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, QuerierWrapper,
    Response, StdResult,
};
use shade_oracles::{
    core::{
        admin::helpers::{validate_admin, AdminPermissions},
        pad_handle_result, pad_query_result,
        ssp::ItemStorage,
        Contract,
    },
    create_attr_action,
    interfaces::{
        band::{reference_data, reference_data_bulk, ReferenceData},
        common::{OraclePrice, PriceResponse, PricesResponse},
        router::{error::*, msg::*, registry::*},
    },
    querier::{query_price, query_prices},
    status::GlobalStatus,
    BLOCK_SIZE,
};

create_attr_action!("oracle-router_");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> OracleRouterResult<Response> {
    let config = Config {
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
        this: Contract::new(&env.contract.address, &env.contract.code_hash),
        band: msg.band.into_valid(deps.api)?,
        quote_symbol: msg.quote_symbol,
    };
    OracleRouter::init_status(deps.storage)?;
    KEYS.save(deps.storage, &vec![])?;
    config.save(deps.storage)?;
    Ok(Response::default().add_attributes(vec![attr_action!("instantiate")]))
}

fn require_admin(
    router: &OracleRouter,
    querier: &QuerierWrapper,
    sender: &Addr,
) -> OracleRouterResult<()> {
    validate_admin(
        querier,
        AdminPermissions::OraclesAdmin,
        sender,
        &router.config.admin_auth,
    )?;
    Ok(())
}

fn require_bot(
    router: &OracleRouter,
    querier: &QuerierWrapper,
    sender: &Addr,
) -> OracleRouterResult<()> {
    validate_admin(
        querier,
        AdminPermissions::OraclesPriceBot,
        sender,
        &router.config.admin_auth,
    )?;
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> OracleRouterResult<Response> {
    let router = OracleRouter::load(deps.storage)?;

    let result = match msg {
        ExecuteMsg::SetStatus { status } => {
            require_admin(&router, &deps.querier, &info.sender)?;
            OracleRouter::update_status(deps.storage, status)?;
            Ok(Response::default().add_attributes(vec![attr_action!("set_status")]))
        }
        _ => {
            OracleRouter::require_can_run(deps.storage, true, true, false)?;
            match msg {
                ExecuteMsg::UpdateConfig { config } => {
                    require_admin(&router, &deps.querier, &info.sender)?;
                    router
                        .update_config(deps.api, config)?
                        .config
                        .save(deps.storage)?;
                    Ok(Response::new().add_attributes(vec![attr_action!("update_config")]))
                }
                ExecuteMsg::UpdateRegistry { operation } => {
                    require_admin(&router, &deps.querier, &info.sender)?;
                    OracleRouter::resolve_registry_operation(deps.api, deps.storage, operation)?;
                    Ok(Response::new().add_attributes(vec![attr_action!("update_registry")]))
                }
                ExecuteMsg::BatchUpdateRegistry { operations } => {
                    require_admin(&router, &deps.querier, &info.sender)?;
                    for operation in operations {
                        OracleRouter::resolve_registry_operation(
                            deps.api,
                            deps.storage,
                            operation,
                        )?;
                    }
                    Ok(Response::new().add_attributes(vec![attr_action!("batch_update_registry")]))
                }
                ExecuteMsg::UpdateProtectedKeys { prices } => {
                    require_bot(&router, &deps.querier, &info.sender)?;
                    for (key, price) in prices {
                        OracleRouter::update_protected_key(deps.storage, &key, price)?;
                    }
                    Ok(Response::new().add_attributes(vec![attr_action!("update_protected_keys")]))
                }
                ExecuteMsg::SetStatus { .. } => panic!("Code should never get here."),
            }
        }
    };
    Ok(pad_handle_result(result, BLOCK_SIZE)?)
}

/// Queries the oracle at the key, if no oracle exists at the key, queries the default oracle.
pub fn get_price(deps: Deps, router: OracleRouter, key: String) -> StdResult<PriceResponse> {
    let oracle = router.get_oracle(deps.storage, &key)?;
    let price = if oracle.eq(&router.config.this) {
        query_band_price(deps, &router, key)
    } else {
        query_price(&oracle, &deps.querier, &key)
    }?;
    OracleRouter::try_deviation_test(deps.storage, &price)?;
    Ok(price)
}

/// Builds bulk queries using the keys given.
pub fn get_prices(
    deps: Deps,
    router: OracleRouter,
    keys: Vec<String>,
) -> StdResult<PricesResponse> {
    let map = router.group_keys_by_oracle(deps.storage, keys.as_slice())?;
    // Preserve symbol order
    let mut prices: Vec<OraclePrice> = keys
        .iter()
        .map(|key| OraclePrice::new(key.clone(), ReferenceData::default()))
        .collect();
    // Temp vector of fetched prices
    let mut unordered_prices = vec![];

    for (oracle, symbols) in map {
        let mut queried_prices: PricesResponse = if oracle.eq(&router.config.this) {
            query_band_prices(deps, &router, symbols)
        } else {
            query_prices(&oracle, &deps.querier, &symbols)
        }?;
        unordered_prices.append(&mut queried_prices);
    }

    // For every fetched price, find its position in the original request and replace the placeholder data with the actual data for that symbol.
    for queried_price in unordered_prices {
        OracleRouter::try_deviation_test(deps.storage, &queried_price)?;
        let position = prices
            .iter()
            .position(|price| price.key.eq(queried_price.key()));
        if let Some(index) = position {
            _ = std::mem::replace(&mut prices[index], queried_price);
        }
    }
    Ok(prices)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let router = OracleRouter::load(deps.storage)?;
    let resp = match msg {
        QueryMsg::GetConfig {} => to_binary(&ConfigResponse {
            config: router.config,
            status: OracleRouter::load_status(deps.storage)?,
        }),
        _ => {
            // If deprecated or frozen, throw error so dependencies cannot query it.
            OracleRouter::require_can_run(deps.storage, true, false, false)?;
            match msg {
                QueryMsg::GetOracle { key } => {
                    let oracle = router.get_oracle(deps.storage, &key)?;
                    to_binary(&OracleResponse { oracle, key })
                }
                QueryMsg::GetPrice { key } => to_binary(&get_price(deps, router, key)?),
                QueryMsg::GetOracles { keys } => {
                    let mut oracles = vec![];
                    for key in keys {
                        let oracle = router.get_oracle(deps.storage, &key)?;
                        oracles.push(OracleResponse { key, oracle })
                    }
                    to_binary(&OraclesResponse { oracles })
                }
                QueryMsg::GetPrices { keys } => to_binary(&get_prices(deps, router, keys)?),
                QueryMsg::GetKeys {} => Ok(OracleRouter::get_keys(deps)?),
                _ => panic!("Code should never go here."),
            }
        }
    };
    pad_query_result(resp, BLOCK_SIZE)
}

fn query_band_price(deps: Deps, router: &OracleRouter, key: String) -> StdResult<PriceResponse> {
    let band_response = reference_data(
        &deps.querier,
        key.clone(),
        router.config.quote_symbol.clone(),
        &router.config.band,
    )?;
    Ok(OraclePrice::new(key, band_response))
}

fn query_band_prices(
    deps: Deps,
    router: &OracleRouter,
    keys: Vec<String>,
) -> StdResult<PricesResponse> {
    let quote_symbol = router.config.quote_symbol.clone();
    let quote_symbols = vec![quote_symbol; keys.len()];
    let band = &router.config.band;

    let band_response = reference_data_bulk(&deps.querier, keys.clone(), quote_symbols, band)?;

    let mut prices: Vec<OraclePrice> = vec![];
    for (index, key) in keys.iter().enumerate() {
        prices.push(OraclePrice::new(
            key.to_string(),
            band_response[index].clone(),
        ));
    }
    Ok(prices)
}
