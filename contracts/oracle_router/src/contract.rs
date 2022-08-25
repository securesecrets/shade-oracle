use cosmwasm_std::{entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use shade_oracles::{
    common::{
        querier::{query_oracle_price, query_oracle_prices},
        PriceResponse, PricesResponse, ShadeOraclePermissions,
    },
    core::{
        mulberry::{common::GlobalStatus, create_attr_action},
        pad_handle_result, pad_query_result,
        ssp::ItemStorage,
        validate_permission,
    },
    interfaces::router::{error::*, msg::*, registry::*},
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
        admin_auth: msg.admin_auth,
        default_oracle: msg.default_oracle,
        address: env.contract.address,
        band: msg.band,
        quote_symbol: msg.quote_symbol,
    };
    OracleRouter::init_status(deps.storage)?;
    config.save(deps.storage)?;
    Ok(Response::default().add_attributes(vec![attr_action!("instantiate")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> OracleRouterResult<Response> {
    let router = OracleRouter::load(deps.storage)?;
    // Ensure sender is admin
    validate_permission(
        &deps.querier,
        &ShadeOraclePermissions::SuperAdmin.to_string(),
        &info.sender,
        &router.config.admin_auth,
    )?;

    let result = match msg {
        ExecuteMsg::SetStatus { status } => {
            OracleRouter::update_status(deps.storage, status)?;
            Ok(Response::default().add_attributes(vec![attr_action!("set_status")]))
        }
        _ => {
            router.require_can_run(deps.storage, true, true, false)?;
            match msg {
                ExecuteMsg::UpdateConfig { config } => {
                    router.update_config(config).config.save(deps.storage)?;
                    Ok(Response::new().add_attributes(vec![attr_action!("update_config")]))
                }
                ExecuteMsg::UpdateRegistry { operation } => {
                    OracleRouter::resolve_registry_operation(deps.storage, operation)?;
                    Ok(Response::new().add_attributes(vec![attr_action!("update_registry")]))
                }
                ExecuteMsg::BatchUpdateRegistry { operations } => {
                    for operation in operations {
                        OracleRouter::resolve_registry_operation(deps.storage, operation)?;
                    }
                    Ok(Response::new().add_attributes(vec![attr_action!("batch_update_registry")]))
                }
                ExecuteMsg::SetStatus { .. } => panic!("Code should never get here."),
            }
        }
    };
    Ok(pad_handle_result(result, BLOCK_SIZE)?)
}

/// Queries the oracle at the key, if no oracle exists at the key, queries the default oracle.
pub fn get_price(deps: Deps, router: OracleRouter, key: String) -> OracleRouterResult<Binary> {
    let oracle = router.get_oracle(deps.storage, &key)?;
    Ok(to_binary(&PriceResponse {
        price: query_oracle_price(&oracle, &deps.querier, &key)?,
    })?)
}

/// Builds bulk queries using the keys given. Does not preserve order.
pub fn get_prices(
    deps: Deps,
    router: OracleRouter,
    keys: Vec<String>,
) -> OracleRouterResult<Binary> {
    let map = router.group_keys_by_oracle(deps.storage, keys.as_slice())?;
    let mut prices = vec![];

    for (key, value) in map {
        if value.len() == 1 {
            let queried_price = query_oracle_price(&key, &deps.querier, value[0].clone())?;
            prices.push(queried_price);
        } else {
            let mut queried_prices = query_oracle_prices(&key, &deps.querier, value)?;
            prices.append(&mut queried_prices);
        }
    }

    Ok(to_binary(&PricesResponse { prices })?)
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> OracleRouterResult<Binary> {
    let router = OracleRouter::load(deps.storage)?;
    let resp = match msg {
        QueryMsg::GetConfig {} => to_binary(&ConfigResponse {
            config: router.config,
            status: OracleRouter::load_status(deps.storage)?,
        }),
        _ => {
            // If deprecated or frozen, throw error so dependencies cannot query it.
            router.require_can_run(deps.storage, true, false, false)?;
            match msg {
                QueryMsg::GetOracle { key } => {
                    let oracle = router.get_oracle(deps.storage, &key)?;
                    to_binary(&OracleResponse { oracle, key })
                }
                QueryMsg::GetPrice { key } => Ok(get_price(deps, router, key)?),
                QueryMsg::GetOracles { keys } => {
                    let mut oracles = vec![];
                    for key in keys {
                        let oracle = router.get_oracle(deps.storage, &key)?;
                        oracles.push(OracleResponse { key, oracle })
                    }
                    to_binary(&oracles)
                }
                QueryMsg::GetPrices { keys } => Ok(get_prices(deps, router, keys)?),
                _ => panic!("Code should never go here."),
            }
        }
    };
    Ok(pad_query_result(resp, BLOCK_SIZE)?)
}
