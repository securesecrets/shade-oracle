use mulberry_utils::{
    common::querier::query_token_info,
    common::types::{CanonicalContract, Contract, ResponseStatus},
    protocols::secretswap::{
        AssetInfo, SecretSwapPairInfo, SecretSwapPairQueryMsg, SecretSwapPoolResponse,
    },
    scrt::{
        to_binary, Api, CanonicalAddr, Env, Extern, HandleResponse, HumanAddr, InitResponse,
        Querier, QueryRequest, QueryResult, StdError, StdResult, Storage, Uint128, WasmQuery,
        BLOCK_SIZE,
    },
    secret_toolkit::utils::{pad_handle_result, pad_query_result},
    storage::traits::SingletonStorable,
};
use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::{PriceResponse, QueryMsg},
    lp::{
        get_fair_lp_token_price,
        secretswap::{ConfigResponse, HandleAnswer, HandleMsg, InitMsg},
        FairLpPriceInfo,
    },
    router::querier::query_price,
};
use std::cmp::min;

/// state of the auction
#[derive(Serialize, Deserialize)]
pub struct State {
    pub owner: CanonicalAddr,
    pub symbol_0: String,
    pub symbol_1: String,
    pub router: CanonicalContract,
    pub factory: CanonicalContract,
    pub lp_token: CanonicalContract,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
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

    let factory: CanonicalContract = CanonicalContract {
        address: deps
            .api
            .canonical_address(&HumanAddr(msg.factory.address.clone()))?,
        code_hash: msg.factory.code_hash.clone(),
    };

    let mut token0: Contract = Contract {
        address: "a".to_string(),
        code_hash: "b".to_string(),
    };
    let mut token1: Contract = Contract {
        address: "a".to_string(),
        code_hash: "b".to_string(),
    };

    let pair_info: SecretSwapPairInfo =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: HumanAddr::from(msg.factory.address),
            callback_code_hash: msg.factory.code_hash,
            msg: to_binary(&SecretSwapPairQueryMsg::Pair {})?,
        }))?;
    let lp_token = CanonicalContract {
        address: deps.api.canonical_address(&pair_info.liquidity_token)?,
        code_hash: pair_info.token_code_hash,
    };

    if let AssetInfo::Token {
        contract_addr,
        token_code_hash,
        viewing_key: _,
    } = &pair_info.asset_infos[0]
    {
        token0.address = contract_addr.to_string();
        token0.code_hash = token_code_hash.to_string();
    } else {
        return Err(StdError::generic_err(
            "Could not resolve SecretSwap token 1 info.",
        ));
    }

    if let AssetInfo::Token {
        contract_addr,
        token_code_hash,
        viewing_key: _,
    } = &pair_info.asset_infos[1]
    {
        token1.address = contract_addr.to_string();
        token1.code_hash = token_code_hash.to_string();
    } else {
        return Err(StdError::generic_err(
            "Could not resolve SecretSwap token 2 info.",
        ));
    }

    let token0_decimals = query_token_info(&token0, &deps.querier)?
        .token_info
        .decimals;
    let token1_decimals = query_token_info(&token1, &deps.querier)?
        .token_info
        .decimals;

    let state: State = State {
        owner: deps.api.canonical_address(&HumanAddr(msg.owner))?,
        symbol_0: msg.symbol_0,
        symbol_1: msg.symbol_1,
        router,
        factory,
        lp_token,
        token0_decimals,
        token1_decimals,
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
                symbol_0,
                symbol_1,
                router,
                factory,
            } => try_update_config(deps, env, owner, symbol_0, symbol_1, router, factory),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<String>,
    symbol_0: Option<String>,
    symbol_1: Option<String>,
    router: Option<Contract>,
    factory: Option<Contract>,
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

    if let Some(symbol_0) = symbol_0 {
        state.symbol_0 = symbol_0;
    }

    if let Some(symbol_1) = symbol_1 {
        state.symbol_1 = symbol_1;
    }

    if let Some(factory) = factory {
        let factory = CanonicalContract {
            address: deps.api.canonical_address(&HumanAddr(factory.address))?,
            code_hash: factory.code_hash,
        };
        state.factory = factory;
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
        symbol_0: state.symbol_0,
        symbol_1: state.symbol_1,
        router: state.router.as_human(&deps.api)?,
        factory: state.factory.as_human(&deps.api)?,
    })
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<PriceResponse> {
    let state: State = State::new_json(&deps.storage)?;

    let price0: PriceResponse = query_price(
        &state.router.as_human(&deps.api)?,
        &deps.querier,
        state.symbol_0,
    )?;

    let price1: PriceResponse = query_price(
        &state.router.as_human(&deps.api)?,
        &deps.querier,
        state.symbol_1,
    )?;

    let pair_info: SecretSwapPoolResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&state.factory.address)?,
            callback_code_hash: state.factory.code_hash,
            msg: to_binary(&SecretSwapPairQueryMsg::Pool {})?,
        }))?;
    let reserve0 = pair_info.assets[0].amount;
    let reserve1 = pair_info.assets[1].amount;

    let lp_token_info = query_token_info(&state.lp_token.as_human(&deps.api)?, &deps.querier)?;

    let total_supply = lp_token_info.token_info.total_supply.unwrap();
    let lp_token_decimals = lp_token_info.token_info.decimals;

    let a = FairLpPriceInfo {
        reserve: reserve0.u128(),
        price: price0.rate.u128(),
        decimals: state.token0_decimals,
    };

    let b = FairLpPriceInfo {
        reserve: reserve1.u128(),
        price: price1.rate.u128(),
        decimals: state.token1_decimals,
    };

    let price = get_fair_lp_token_price(a, b, total_supply.u128(), lp_token_decimals);

    let response = PriceResponse {
        rate: Uint128(price.unwrap()),
        last_updated_base: min(price0.last_updated_base, price1.last_updated_base),
        last_updated_quote: min(price0.last_updated_quote, price1.last_updated_quote),
    };
    Ok(response)
}
