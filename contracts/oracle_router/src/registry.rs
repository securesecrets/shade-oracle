use std::collections::HashMap;

use crate::{contract::get_oracle, state::*};
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Deps, Response, Querier, StdError, StdResult, Storage,
};
use shade_oracles::{
    common::{
        querier::{query_oracle_price, query_oracle_prices},
        Contract, OraclePrice,
    },
    router::*,
};

pub fn update_registry(
    deps: DepsMut,
    _env: Env,
    operation: RegistryOperation,
) -> StdResult<Response> {
    resolve_registry_operation(&mut deps.storage, &deps.api, operation)?;
    Ok(Response::default())
}

pub fn batch_update_registry(
    deps: DepsMut,
    _env: Env,
    operations: Vec<RegistryOperation>,
) -> StdResult<Response> {
    for operation in operations {
        resolve_registry_operation(&mut deps.storage, &deps.api, operation)?;
    }
    Ok(Response::default())
}

/// Queries the oracle at the key, if no oracle exists at the key, queries the default oracle.
pub fn get_price(
    deps: Deps,
    key: String,
) -> StdResult<Binary> {
    let resolved_key = resolve_alias(deps.storage, key)?;
    let oracle = get_oracle(deps.storage, &resolved_key)?;

    to_binary(&query_oracle_price(&oracle, &deps.querier, resolved_key)?)
}

/// Builds bulk queries using the keys given.
pub fn get_prices(
    deps: Deps,
    keys: Vec<String>,
) -> StdResult<Binary> {
    // Maps oracle to the symbols it is responsible for
    let mut map: HashMap<Contract, Vec<String>> = HashMap::new();

    for current_key in keys {
        let resolved_key = resolve_alias(deps.storage, current_key.clone())?;
        let oracle = get_oracle(deps.storage, &resolved_key)?;

        // Get the current vector of symbols at that oracle and add the current key to it
        map.entry(oracle).or_insert(vec![]).push(resolved_key);
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
    to_binary(&prices)
}

pub fn resolve_alias(storage: &impl Storage, alias: String) -> StdResult<String> {
    match ALIASES.may_load(storage, alias.clone()) {
        Ok(key) => match key {
            Some(key) => Ok(key),
            None => Ok(alias),
        },
        Err(_) => Err(StdError::generic_err(
            "Failed to fetch from the ALIASES storage.",
        )),
    }
}

fn resolve_registry_operation(
    storage: &mut impl Storage,
    _api: &impl Api,
    operation: RegistryOperation,
) -> StdResult<()> {
    match operation {
        RegistryOperation::Remove { key } => {
            ORACLES.remove(storage, key);
            Ok(())
        }
        RegistryOperation::Replace { oracle, key } => {
            ORACLES.update(storage, key, |_| -> StdResult<_> { Ok(oracle) })?;
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
            Ok(())
        }
        RegistryOperation::UpdateAlias { alias, key } => {
            ALIASES.update(storage, alias, |old_key| -> StdResult<_> {
                match old_key {
                    Some(_) => Ok(key),
                    None => Ok(key),
                }
            })?;
            Ok(())
        }
    }
}
