use mulberry_utils::{
    common::querier::query_token_info,
    common::types::{CanonicalContract, Contract, ResponseStatus},
    get_precision,
    scrt::{
        to_binary, Api, CanonicalAddr, Env, Extern, HandleResponse, HumanAddr, InitResponse,
        Querier, QueryResult, StdError, StdResult, Storage, Uint128, BLOCK_SIZE,
    },
    secret_toolkit::utils::{pad_handle_result, pad_query_result},
    storage::traits::SingletonStorable,
};
use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::{query_price, PriceResponse, QueryMsg},
    router::querier::query_oracle,
    staking_derivative::shade::{
        querier::query_price as query_derivative_price, ConfigResponse, HandleAnswer, HandleMsg,
        InitMsg,
    },
};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub owner: CanonicalAddr,
    pub symbol: String,
    pub router: CanonicalContract,
    pub staking_derivative: CanonicalContract,
    pub token_decimals: u8,
}

impl SingletonStorable for State {
    fn namespace() -> Vec<u8> {
        b"config".to_vec()
    }
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let router: CanonicalContract = CanonicalContract {
        address: deps.api.canonical_address(&HumanAddr(msg.router.address))?,
        code_hash: msg.router.code_hash,
    };

    let staking_derivative: CanonicalContract = CanonicalContract {
        address: deps
            .api
            .canonical_address(&HumanAddr(msg.staking_derivative.address.clone()))?,
        code_hash: msg.staking_derivative.code_hash.clone(),
    };

    let token_decimals = query_token_info(&msg.staking_derivative, &deps.querier)?
        .token_info
        .decimals;

    let state: State = State {
        owner: deps.api.canonical_address(&HumanAddr(msg.owner))?,
        symbol: msg.symbol,
        router,
        staking_derivative,
        token_decimals,
    };

    state.save_json(&mut deps.storage)?;

    Ok(InitResponse::default())
}

/* CONFIG UPDATE NEEDS TO BE FIXED */
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    pad_handle_result(
        match msg {
            HandleMsg::UpdateConfig {
                owner,
                symbol,
                staking_derivative,
                router,
            } => try_update_config(deps, env, owner, symbol, staking_derivative, router),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<String>,
    symbol: Option<String>,
    staking_derivative: Option<Contract>,
    router: Option<Contract>,
) -> StdResult<HandleResponse> {
    let mut state: State = State::new_json(&deps.storage)?;

    if deps.api.canonical_address(&env.message.sender)? != state.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    if let Some(owner) = owner {
        state.owner = deps.api.canonical_address(&HumanAddr(owner))?;
    }

    if let Some(router) = router {
        let router = CanonicalContract {
            address: deps.api.canonical_address(&HumanAddr(router.address))?,
            code_hash: router.code_hash,
        };
        state.router = router;
    }

    if let Some(staking_derivative) = staking_derivative {
        let staking_derivative = CanonicalContract {
            address: deps
                .api
                .canonical_address(&HumanAddr(staking_derivative.address))?,
            code_hash: staking_derivative.code_hash,
        };
        state.staking_derivative = staking_derivative;
    }

    if let Some(symbol) = symbol {
        state.symbol = symbol;
    }

    state.save_json(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => to_binary(&try_query_config(deps)?),
            QueryMsg::GetPrice { .. } => to_binary(&try_query_price(deps)?),
        },
        BLOCK_SIZE,
    )
}

fn try_query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let state: State = State::new_json(&deps.storage)?;

    Ok(ConfigResponse {
        owner: deps.api.human_address(&state.owner)?.to_string(),
        symbol: state.symbol,
        router: state.router.as_human(&deps.api)?,
        staking_derivative: state.staking_derivative.as_human(&deps.api)?,
    })
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<PriceResponse> {
    let state: State = State::new_json(&deps.storage)?;

    let underlying_oracle = query_oracle(
        &state.router.as_human(&deps.api)?,
        &deps.querier,
        state.symbol,
    )?;

    // price of underlying asset to 10^18.
    let underlying_price = query_price(&underlying_oracle, &deps.querier)?;

    let staking_derivative_price = query_derivative_price(
        &state.staking_derivative.as_human(&deps.api)?,
        &deps.querier,
    )?;

    let staking_derivative_price_precision =
        Uint128(get_precision(state.token_decimals).clamp_u128()?);

    let price = underlying_price
        .rate
        .multiply_ratio(staking_derivative_price, staking_derivative_price_precision);

    let response = PriceResponse {
        rate: price,
        last_updated_base: underlying_price.last_updated_base,
        last_updated_quote: underlying_price.last_updated_quote,
    };
    Ok(response)
}
