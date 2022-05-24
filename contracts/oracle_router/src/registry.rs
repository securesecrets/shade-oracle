use std::collections::HashMap;

use crate::state::*;
use shade_oracles::common::{Contract, OraclePrice};
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage,
};
use shade_oracles::{common::querier::{query_price, query_prices}, router::*};

pub fn update_registry<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    operation: RegistryOperation,
) -> StdResult<HandleResponse> {
    resolve_registry_operation(&mut deps.storage, &deps.api, operation)?;
    Ok(HandleResponse::default())
}

pub fn batch_update_registry<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    operations: Vec<RegistryOperation>,
) -> StdResult<HandleResponse> {
    for operation in operations {
        resolve_registry_operation(&mut deps.storage, &deps.api, operation)?;
    }
    Ok(HandleResponse::default())
}

/// Queries the oracle at the key, if no oracle exists at the key, queries the default oracle.
pub fn get_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: String,
) -> StdResult<Binary> {
    let oracle = match ORACLES.may_load(&deps.storage, key.clone())? {
        Some(oracle) => oracle,
        None => CONFIG.load(&deps.storage)?.default_oracle,
    };

    to_binary(&query_price(&oracle, &deps.querier, key)?)
}

/// Builds bulk queries using the keys given.
pub fn get_prices<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    keys: Vec<String>,
) -> StdResult<Binary> {
    // Maps oracle to the symbols it is responsible for
    let mut map: HashMap<Contract, Vec<String>> = HashMap::new();

    for current_key in keys {
        let oracle = match ORACLES.may_load(&deps.storage, current_key.clone())? {
            Some(oracle) => oracle,
            None => CONFIG.load(&deps.storage)?.default_oracle,
        };
        
        // Get the current vector of symbols at that oracle and add the current key to it
        map.entry(oracle).or_insert(vec![]).push(current_key);
    }

    let mut prices: Vec<OraclePrice> = vec![];

    for (key, value) in map {
        let mut queried_prices = query_prices(&key, &deps.querier, value)?;
        prices.append(&mut queried_prices);
    }
    to_binary(&prices)
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
        },
        RegistryOperation::Replace { oracle, key } => {
            ORACLES.update(storage, key, |_| -> StdResult<_> {
                Ok(oracle)
            })?;
            Ok(())
        }
        RegistryOperation::Add { oracle, key } => {
            ORACLES.update(storage, key.clone(), |old_oracle| -> StdResult<_> {
                match old_oracle {
                    Some(_) => Err(StdError::generic_err(format!("An oracle already exists at the key - {}.", key))),
                    None => {
                        Ok(oracle)
                    }
                }            
            })?;
            Ok(())
        }
    }
}
