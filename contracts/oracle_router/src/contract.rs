use crate::{
    registry::{batch_update_registry, get_price, update_registry},
    state::*,
};
use shade_oracles::router::*;
use shade_oracles::{
    scrt::{
        to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
        StdError, StdResult, Storage, BLOCK_SIZE,
        secret_toolkit::utils::{pad_handle_result, pad_query_result},
    },
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = RawConfig {
        owner: deps.api.canonical_address(&HumanAddr(msg.owner))?,
    };
    config.save(&mut deps.storage)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    pad_handle_result(
        match msg {
            HandleMsg::ChangeOwner { new_owner } => {
                is_owner(&deps.storage, &deps.api, &env)?;
                let new_config = RawConfig {
                    owner: deps.api.canonical_address(&HumanAddr(new_owner))?,
                };
                new_config.save(&mut deps.storage)?;
                Ok(HandleResponse::default())
            }
            HandleMsg::UpdateRegistry { operation } => {
                is_owner(&deps.storage, &deps.api, &env)?;
                update_registry(deps, env, operation)
            }
            HandleMsg::BatchUpdateRegistry { operations } => {
                is_owner(&deps.storage, &deps.api, &env)?;
                batch_update_registry(deps, env, operations)
            }
        },
        BLOCK_SIZE,
    )
}

fn is_owner(storage: &impl Storage, api: &impl Api, env: &Env) -> StdResult<()> {
    let config = RawConfig::new(storage)?;
    let current_owner = api.human_address(&config.owner)?;
    if env.message.sender != current_owner {
        Err(StdError::unauthorized())
    } else {
        Ok(())
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::GetOwner {} => {
                let config = RawConfig::new(&deps.storage)?;
                to_binary(&ConfigResponse {
                    owner: deps.api.human_address(&config.owner)?.to_string(),
                })
            }
            QueryMsg::GetOracle { key } => {
                let oracle = Oracle::get(&deps.storage, &deps.api, key.as_str())?;
                to_binary(&OracleResponse { oracle })
            }
            QueryMsg::GetPrice { key } => get_price(deps, key),
            QueryMsg::GetOracles { keys } => todo!(),
            QueryMsg::GetPrices { keys } => todo!(),
        },
        BLOCK_SIZE,
    )
}
