use std::collections::HashMap;

use crate::contract::{get_oracle, KEYS, ORACLES};
use cosmwasm_std::{
    to_binary, Api, Binary, Deps, DepsMut, Env, Response, StdError, StdResult, Storage,
};
use shade_oracles::{
    common::{
        querier::{query_oracle_price, query_oracle_prices},
        OraclePrice, PriceResponse, PricesResponse,
    },
    core::Contract,
    interfaces::router::*,
};

pub fn update_registry(
    deps: DepsMut,
    _env: Env,
    operation: RegistryOperation,
) -> StdResult<Response> {
    resolve_registry_operation(deps.storage, deps.api, operation)?;
    Ok(Response::default())
}

pub fn batch_update_registry(
    deps: DepsMut,
    _env: Env,
    operations: Vec<RegistryOperation>,
) -> StdResult<Response> {
    for operation in operations {
        resolve_registry_operation(deps.storage, deps.api, operation)?;
    }
    Ok(Response::default())
}

/// Queries the oracle at the key, if no oracle exists at the key, queries the default oracle.
pub fn get_price(deps: Deps, key: String) -> StdResult<Binary> {
    let oracle = get_oracle(deps.storage, &key)?;

    to_binary(&PriceResponse {
        price: query_oracle_price(&oracle, &deps.querier, key)?,
    })
}

/// Builds bulk queries using the keys given.
pub fn get_prices(deps: Deps, keys: Vec<String>) -> StdResult<Binary> {
    // Maps oracle to the symbols it is responsible for
    let mut map: HashMap<Contract, Vec<String>> = HashMap::new();

    for key in keys {
        let oracle = get_oracle(deps.storage, &key)?;

        // Get the current vector of symbols at that oracle and add the current key to it
        map.entry(oracle).or_insert(vec![]).push(key);
    }

    let mut prices: Vec<OraclePrice> = vec![];

    for (key, value) in map {
        if value.len() == 1 {
            let queried_price = query_oracle_price(&key, &deps.querier, value[0].clone())?;
            prices.push(queried_price);
        } else {
            let mut queried_prices = query_oracle_prices(&key, &deps.querier, value)?;
            prices.append(&mut queried_prices);
        }
    }
    to_binary(&PricesResponse { prices })
}

pub fn get_keys(deps: Deps) -> StdResult<Binary> {
    let keys = KEYS.load(deps.storage)?;
    to_binary(&KeysResponse { keys })
}

fn resolve_registry_operation(
    storage: &mut dyn Storage,
    _api: &dyn Api,
    operation: RegistryOperation,
) -> StdResult<()> {
    match operation {
        RegistryOperation::Remove { key } => {
            ORACLES.remove(storage, key.clone());
            KEYS.update(storage, |mut keys| -> StdResult<_> {
                keys.retain(|k| key.ne(k));
                Ok(keys)
            })?;
            Ok(())
        }
        RegistryOperation::Replace { oracle, key } => {
            ORACLES.update(storage, key.clone(), |old_oracle| -> StdResult<_> {
                match old_oracle {
                    Some(_) => Ok(oracle),
                    None => Err(StdError::generic_err(format!(
                        "Cannot replace oracle at key {} if there wasn't one already there.",
                        key
                    ))),
                }
            })?;
            KEYS.update(storage, |mut keys| -> StdResult<_> {
                let position = keys.iter().position(|k| key.eq(k));
                if position.is_some() {
                    let index = position.unwrap();
                    keys.swap_remove(index);
                    keys.push(key);
                }
                Ok(keys)
            })?;
            Ok(())
        }
        RegistryOperation::Add { oracle, key } => {
            ORACLES.update(storage, key.clone(), |old_oracle| -> StdResult<_> {
                match old_oracle {
                    Some(_) => Err(StdError::generic_err(format!(
                        "An oracle already exists at the key - {}.",
                        key
                    ))),
                    None => Ok(oracle),
                }
            })?;
            KEYS.update(storage, |mut keys| -> StdResult<_> {
                keys.push(key);
                Ok(keys)
            })?;
            Ok(())
        }
    }
}
