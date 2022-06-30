use crate::{
    registry::{batch_update_registry, get_price, get_prices, resolve_alias, update_registry},
    state::*,
};
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
    StdError, StdResult, Storage,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result, Query};
use shade_admin::admin::{QueryMsg as AdminQueryMsg, ValidateAdminPermissionResponse};
use shade_oracles::{
    common::{Contract, BLOCK_SIZE},
    router::*,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config {
        admin_auth: msg.admin_auth,
        default_oracle: msg.default_oracle,
        address: env.contract.address,
        band: msg.band,
        quote_symbol: msg.quote_symbol,
        enabled: true,
    };
    CONFIG.save(&mut deps.storage, &config)?;
    Ok(InitResponse {
        messages: vec![],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    is_admin(deps, env.message.sender.clone())?;
    pad_handle_result(
        match msg {
            HandleMsg::UpdateConfig { config } => {
                CONFIG.update(&mut deps.storage, |mut new_config| -> StdResult<_> {
                    new_config.admin_auth = config.admin_auth.unwrap_or(new_config.admin_auth);
                    new_config.default_oracle =
                        config.default_oracle.unwrap_or(new_config.default_oracle);
                    new_config.band = config.band.unwrap_or(new_config.band);
                    new_config.quote_symbol =
                        config.quote_symbol.unwrap_or(new_config.quote_symbol);
                    Ok(new_config)
                })?;
                Ok(HandleResponse::default())
            }
            HandleMsg::UpdateRegistry { operation } => update_registry(deps, env, operation),
            HandleMsg::BatchUpdateRegistry { operations } => {
                batch_update_registry(deps, env, operations)
            }
        },
        BLOCK_SIZE,
    )
}

fn is_admin<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    user: HumanAddr,
) -> StdResult<()> {
    let config = CONFIG.load(&deps.storage)?;
    let resp: ValidateAdminPermissionResponse = AdminQueryMsg::ValidateAdminPermission {
        contract_address: config.address.to_string(),
        admin_address: user.to_string(),
    }
    .query(
        &deps.querier,
        config.admin_auth.code_hash,
        config.admin_auth.address,
    )?;
    match resp.error_msg {
        Some(err) => Err(StdError::generic_err(err)),
        None => Ok(()),
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
                to_binary(&config)
            }
            QueryMsg::GetOracle { key } => {
                let oracle = get_oracle(&deps.storage, &key)?;
                to_binary(&OracleResponse { oracle, key })
            }
            QueryMsg::GetPrice { key } => get_price(deps, key),
            QueryMsg::GetOracles { keys } => {
                let mut oracles = vec![];
                for key in keys {
                    let oracle = get_oracle(&deps.storage, &key)?;
                    oracles.push(OracleResponse { key, oracle })
                }
                to_binary(&oracles)
            }
            QueryMsg::GetPrices { keys } => get_prices(deps, keys),
            QueryMsg::GetAdminAuth {} => to_binary(&AdminAuthResponse {
                admin_auth: CONFIG.load(&deps.storage)?.admin_auth,
            }),
        },
        BLOCK_SIZE,
    )
}

pub fn get_oracle(storage: &impl Storage, key: &str) -> StdResult<Contract> {
    let config = CONFIG.load(storage)?;
    let resolved_key = resolve_alias(storage, key.to_string())?;
    match ORACLES.may_load(storage, resolved_key)? {
        Some(contract) => Ok(contract),
        None => Ok(config.default_oracle),
    }
}
