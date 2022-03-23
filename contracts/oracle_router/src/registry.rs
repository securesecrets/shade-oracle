use crate::state::*;
use shade_oracles::{common::query_price, router::*};
use mulberry_utils::scrt::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage,
};

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

pub fn get_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: String,
) -> StdResult<Binary> {
    let oracle = Oracle::get(&deps.storage, &deps.api, key.as_str())?;
    to_binary(&query_price(&oracle, &deps.querier)?)
}

fn resolve_registry_operation(
    storage: &mut impl Storage,
    api: &impl Api,
    operation: RegistryOperation,
) -> StdResult<()> {
    match operation {
        RegistryOperation::Remove { key } => Oracle::remove(storage, key.as_str()),
        RegistryOperation::Replace { oracle, key } => {
            Oracle::remove(storage, key.as_str())?;
            let new_oracle = Oracle::new(api, oracle)?;
            new_oracle.save(storage, key.as_str())
        }
        RegistryOperation::Add { oracle, key } => {
            let result = Oracle::get(storage, api, key.as_str());
            match result {
                Ok(result) => Err(StdError::generic_err("A contract already exists here.")),
                Err(_) => {
                    let new_oracle = Oracle::new(api, oracle)?;
                    new_oracle.save(storage, key.as_str())
                }
            }
        }
    }
}
