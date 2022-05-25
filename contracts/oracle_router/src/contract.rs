use crate::{
    registry::{batch_update_registry, get_price, update_registry, get_prices},
    state::*,
};
use shade_oracles::{common::BLOCK_SIZE, router::*};
use cosmwasm_std::{
        to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
        StdError, StdResult, Storage,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config {
        owner: msg.owner,
        default_oracle: msg.default_oracle,
    };
    CONFIG.save(&mut deps.storage, &config)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    pad_handle_result(
        match msg {
            HandleMsg::UpdateConfig { owner, default_oracle } => {
                is_owner(&deps.storage, &env)?;
                CONFIG.update(&mut deps.storage, |mut new_config| -> StdResult<_> {
                    new_config.owner = owner.unwrap_or(new_config.owner);
                    new_config.default_oracle = default_oracle.unwrap_or(new_config.default_oracle);
                    Ok(new_config)
                })?;
                Ok(HandleResponse::default())
            }
            HandleMsg::UpdateRegistry { operation } => {
                is_owner(&deps.storage, &env)?;
                update_registry(deps, env, operation)
            }
            HandleMsg::BatchUpdateRegistry { operations } => {
                is_owner(&deps.storage, &env)?;
                batch_update_registry(deps, env, operations)
            }
        },
        BLOCK_SIZE,
    )
}

fn is_owner(storage: &impl Storage, env: &Env) -> StdResult<()> {
    let config = CONFIG.load(storage)?;
    if env.message.sender != config.owner {
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
            QueryMsg::GetConfig {} => {
                let config = CONFIG.load(&deps.storage)?;
                to_binary(&ConfigResponse {
                    owner: config.owner,
                    default_oracle: config.default_oracle,
                })
            }
            QueryMsg::GetOracle { key } => {
                let oracle = ORACLES.load(&deps.storage, key.clone())?;
                to_binary(&OracleResponse { oracle, key })
            }
            QueryMsg::GetPrice { key } => get_price(deps, key),
            QueryMsg::GetOracles { keys } => {
                let mut oracles = vec![];
                for key in keys {
                    let oracle = ORACLES.load(&deps.storage, key.clone())?;
                    oracles.push(OracleResponse { key, oracle })
                }
                to_binary(&oracles)
            },
            QueryMsg::GetPrices { keys } => get_prices(deps, keys),
        },
        BLOCK_SIZE,
    )
}
