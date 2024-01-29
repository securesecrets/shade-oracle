use std::collections::HashMap;

use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, QuerierWrapper,
    Response, StdResult,
};
use shade_oracles::{
    core::{
        pad_execute_result, pad_query_result, ssp::ItemStorage, validate_admin, AdminPermissions,
        Contract,
    },
    create_attr_action,
    interfaces::{
        common::{OraclePrice, PriceResponse, PricesResponse},
        providers::ReferenceData,
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
        admin_auth: msg.admin_auth.validate(deps.api)?,
        this: Contract::new(env.contract.address.as_str(), &env.contract.code_hash),
        provider: msg.provider.validate(deps.api)?,
        quote_symbol: msg.quote_symbol,
    };
    OracleRouter::init_status(deps.storage)?;
    OracleRouter::init_storage(deps.storage)?;
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
        ExecuteMsg::SetStatus(status) => {
            require_admin(&router, &deps.querier, &info.sender)?;
            OracleRouter::update_status(deps.storage, status)?;
            Ok(Response::default().add_attributes(vec![attr_action!("set_status")]))
        }
        _ => {
            OracleRouter::require_can_run(deps.storage, true, true, false)?;
            match msg {
                ExecuteMsg::UpdateConfig(config) => {
                    require_admin(&router, &deps.querier, &info.sender)?;
                    router
                        .update_config(deps.api, config)?
                        .config
                        .save(deps.storage)?;
                    Ok(Response::new().add_attributes(vec![attr_action!("update_config")]))
                }
                ExecuteMsg::UpdateRegistry(operation) => {
                    require_admin(&router, &deps.querier, &info.sender)?;
                    OracleRouter::resolve_registry_operation(deps.api, deps.storage, operation)?;
                    Ok(Response::new().add_attributes(vec![attr_action!("update_registry")]))
                }
                ExecuteMsg::BatchUpdateRegistry(operations) => {
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
                ExecuteMsg::UpdateProtectedKeys(prices) => {
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
    Ok(pad_execute_result(result, BLOCK_SIZE)?)
}

/// Queries the oracle at the key, if no oracle exists at the key, queries the default oracle.
pub fn get_price(deps: Deps, router: OracleRouter, key: String) -> StdResult<PriceResponse> {
    let oracle = router.get_oracle(deps.storage, &key)?;
    let price = if oracle.eq(&router.config.this) {
        router.query_provider_price(&deps.querier, key)
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
    let ordered_prices: Vec<OraclePrice> = keys
        .iter()
        .map(|key| OraclePrice::new(key.clone(), ReferenceData::default()))
        .collect();
    // Temp vector of fetched prices
    let mut fetched_prices: HashMap<String, OraclePrice> = HashMap::new();

    for (oracle, symbols) in map {
        let queried_prices: PricesResponse = if oracle.eq(&router.config.this) {
            if symbols.len() == 1 {
                let price = router.query_provider_price(&deps.querier, symbols[0].clone())?;
                Ok(vec![price])
            } else {
                router.query_provider_prices(&deps.querier, symbols)
            }
        } else if symbols.len() == 1 {
            let price = query_price(&oracle, &deps.querier, &symbols[0])?;
            Ok(vec![price])
        } else {
            query_prices(&oracle, &deps.querier, &symbols)
        }?;
        for price in queried_prices {
            OracleRouter::try_deviation_test(deps.storage, &price)?;
            if fetched_prices.get(price.key()).is_none() {
                fetched_prices.insert(price.key.clone(), price);
            }
        }
    }

    // For every fetched price, find its position in the original request and replace the placeholder data with the actual data for that symbol.
    let ordered_prices: Vec<OraclePrice> = ordered_prices
        .into_iter()
        .map(|mut p| {
            if let Some(fetched_price) = fetched_prices.get(p.key()) {
                p.data = fetched_price.data.clone();
            }
            p
        })
        .collect();
    Ok(ordered_prices)
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
            match msg {
                QueryMsg::GetOracle { key } => {
                    let oracle = router.get_oracle(deps.storage, &key)?;
                    to_binary(&OracleResponse { oracle, key })
                }
                QueryMsg::GetPrice { key } => {
                    // If deprecated or frozen, throw error so dependencies cannot query it.
                    OracleRouter::require_can_run(deps.storage, true, false, false)?;
                    to_binary(&get_price(deps, router, key)?)
                }
                QueryMsg::GetOracles { keys } => {
                    let mut oracles = vec![];
                    for key in keys {
                        let oracle = router.get_oracle(deps.storage, &key)?;
                        oracles.push(OracleResponse { key, oracle })
                    }
                    to_binary(&oracles)
                }
                QueryMsg::GetPrices { keys } => {
                    // If deprecated or frozen, throw error so dependencies cannot query it.
                    OracleRouter::require_can_run(deps.storage, true, false, false)?;
                    to_binary(&get_prices(deps, router, keys)?)
                }
                QueryMsg::GetKeys {} => Ok(OracleRouter::get_keys(deps)?),
                QueryMsg::GetProtectedKeys {} => Ok(OracleRouter::get_protected_keys(deps)?),
                _ => panic!("Code should never go here."),
            }
        }
    };
    pad_query_result(resp, BLOCK_SIZE)
}
