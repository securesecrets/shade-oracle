use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::querier::query_token_info,
    common::types::{CanonicalContract, Contract, ResponseStatus},
    protocols::secretswap::{
        AssetInfo, SecretSwapPairInfo, SecretSwapPairQueryMsg, SecretSwapPoolResponse,
    },
    scrt::{
        to_binary, Api, CanonicalAddr, Env, Deps, Response, Addr, InitResponse,
        Querier, QueryRequest, StdError, StdResult, Storage, Uint128, WasmQuery,
        BLOCK_SIZE,
    },
    secret_toolkit::utils::{pad_handle_result, pad_query_result},
    storage::traits::SingletonStorable,
};
use shade_oracles::{
    common::{query_price, PriceResponse, QueryMsg},
    lp::{
        get_fair_lp_token_price,
        secretswap::{ConfigResponse, HandleAnswer, ExecuteMsg, InstantiateMsg},
        FairLpPriceInfo,
    },
    router::querier::query_oracle,
};
use std::cmp::min;

/// state of the auction
#[derive(Serialize, Deserialize)]
pub struct State {
    pub owner: CanonicalAddr,
    pub symbol_0: String,
    pub symbol_1: String,
    pub router: CanonicalContract,
    pub pair: CanonicalContract,
    pub lp_token: CanonicalContract,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
}

impl SingletonStorable for State {
    fn namespace() -> Vec<u8> {
        b"config".to_vec()
    }
}

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<InitResponse> {
    let router: CanonicalContract = CanonicalContract {
        address: deps.api.canonical_address(&Addr(msg.router.address))?,
        code_hash: msg.router.code_hash,
    };

    let pair: CanonicalContract = CanonicalContract {
        address: deps
            .api
            .canonical_address(&Addr(msg.pair.address.clone()))?,
        code_hash: msg.pair.code_hash.clone(),
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
            contract_addr: Addr::from(msg.pair.address),
            code_hash: msg.pair.code_hash,
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
        owner: deps.api.canonical_address(&Addr(msg.owner))?,
        symbol_0: msg.symbol_0,
        symbol_1: msg.symbol_1,
        router,
        pair,
        lp_token,
        token0_decimals,
        token1_decimals,
    };

    state.save_json(&mut deps.storage)?;

    Ok(InitResponse::default())
}

/* CONFIG UPDATE NEEDS TO BE FIXED */
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateConfig {
                owner,
                symbol_0,
                symbol_1,
                router,
                pair,
            } => try_update_config(deps, env, owner, symbol_0, symbol_1, router, pair),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config(
    deps: DepsMut,
    env: Env,
    owner: Option<String>,
    symbol_0: Option<String>,
    symbol_1: Option<String>,
    router: Option<Contract>,
    pair: Option<Contract>,
) -> StdResult<Response> {
    let mut state: State = State::new_json(deps.storage)?;

    if deps.api.canonical_address(&env.message.sender)? != state.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    if let Some(owner) = owner {
        state.owner = deps.api.canonical_address(&Addr(owner))?;
    }

    if let Some(router) = router {
        let router = CanonicalContract {
            address: deps.api.canonical_address(&Addr(router.address))?,
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

    if let Some(pair) = pair {
        let pair = CanonicalContract {
            address: deps.api.canonical_address(&Addr(pair.address))?,
            code_hash: pair.code_hash,
        };
        state.pair = pair;
    }

    state.save_json(&mut deps.storage)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => to_binary(&try_query_config(deps)?),
            QueryMsg::GetPrice { .. } => to_binary(&try_query_price(deps)?),
        },
        BLOCK_SIZE,
    )
}

fn try_query_config(
    deps: Deps,
) -> StdResult<ConfigResponse> {
    let state: State = State::new_json(deps.storage)?;

    Ok(ConfigResponse {
        owner: deps.api.human_address(&state.owner)?.to_string(),
        symbol_0: state.symbol_0,
        symbol_1: state.symbol_1,
        router: state.router.as_human(&deps.api)?,
        pair: state.pair.as_human(&deps.api)?,
    })
}

fn try_query_price(
    deps: Deps,
) -> StdResult<PriceResponse> {
    let state: State = State::new_json(deps.storage)?;

    let oracle0 = query_oracle(
        &state.router.as_human(&deps.api)?,
        &deps.querier,
        state.symbol_0,
    )?;
    let oracle1 = query_oracle(
        &state.router.as_human(&deps.api)?,
        &deps.querier,
        state.symbol_1,
    )?;

    let price0: PriceResponse = query_price(&oracle0, &deps.querier)?;

    let price1: PriceResponse = query_price(&oracle1, &deps.querier)?;

    let pair_info: SecretSwapPoolResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&state.pair.address)?,
            code_hash: state.pair.code_hash,
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
