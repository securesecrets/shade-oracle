use crate::registry::{batch_update_registry, get_keys, get_price, get_prices, update_registry};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Storage,
};
use shade_oracles::{
    common::SHADE_ORACLE_ADMIN_PERMISSION,
    core::{pad_handle_result, pad_query_result, validate_permission, Contract},
    interfaces::router::*,
    ssp::{Item, Map},
    BLOCK_SIZE,
};

pub const CONFIG: Item<Config> = Item::new("YteGsgSZyO");
pub const ORACLES: Map<String, Contract> = Map::new("d3a17d1b");
pub const KEYS: Item<Vec<String>> = Item::new("iaunwdioafj");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        admin_auth: msg.admin_auth,
        default_oracle: msg.default_oracle,
        address: env.contract.address,
        band: msg.band,
        quote_symbol: msg.quote_symbol,
        enabled: true,
    };
    KEYS.save(deps.storage, &vec![])?;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    is_admin(deps.as_ref(), info.sender, &env)?;
    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateConfig { config } => {
                CONFIG.update(deps.storage, |mut new_config| -> StdResult<_> {
                    new_config.admin_auth = config.admin_auth.unwrap_or(new_config.admin_auth);
                    new_config.default_oracle =
                        config.default_oracle.unwrap_or(new_config.default_oracle);
                    new_config.band = config.band.unwrap_or(new_config.band);
                    new_config.quote_symbol =
                        config.quote_symbol.unwrap_or(new_config.quote_symbol);
                    Ok(new_config)
                })?;
                Ok(Response::default())
            }
            ExecuteMsg::UpdateRegistry { operation } => update_registry(deps, env, operation),
            ExecuteMsg::BatchUpdateRegistry { operations } => {
                batch_update_registry(deps, env, operations)
            }
        },
        BLOCK_SIZE,
    )
}

fn is_admin(deps: Deps, user: Addr, _env: &Env) -> StdResult<()> {
    let config = CONFIG.load(deps.storage)?;
    validate_permission(
        &deps.querier,
        SHADE_ORACLE_ADMIN_PERMISSION,
        &user,
        &config.admin_auth,
    )
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => {
                let config = CONFIG.load(deps.storage)?;
                to_binary(&config)
            }
            QueryMsg::GetOracle { key } => {
                let oracle = get_oracle(deps.storage, &key)?;
                to_binary(&OracleResponse { oracle, key })
            }
            QueryMsg::GetPrice { key } => get_price(deps, key),
            QueryMsg::GetOracles { keys } => {
                let mut oracles = vec![];
                for key in keys {
                    let oracle = get_oracle(deps.storage, &key)?;
                    oracles.push(OracleResponse { key, oracle })
                }
                to_binary(&oracles)
            }
            QueryMsg::GetPrices { keys } => get_prices(deps, keys),
            QueryMsg::GetAdminAuth {} => to_binary(&AdminAuthResponse {
                admin_auth: CONFIG.load(deps.storage)?.admin_auth,
            }),
            QueryMsg::GetKeys => get_keys(deps),
        },
        BLOCK_SIZE,
    )
}

pub fn get_oracle(storage: &dyn Storage, key: &str) -> StdResult<Contract> {
    let config = CONFIG.load(storage)?;
    match ORACLES.may_load(storage, key.to_string())? {
        Some(contract) => Ok(contract),
        None => Ok(config.default_oracle),
    }
}
