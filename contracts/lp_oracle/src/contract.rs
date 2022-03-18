use serde::{Deserialize, Serialize};
use shade_lend::oracles::{
    common::{query_price, PriceResponse, QueryMsg},
    lp_oracle::{
        get_fair_lp_token_price, ConfigResponse, Dex, FairLpPriceInfo, HandleAnswer, HandleMsg,
        InitMsg,
    },
};
use shared_types::{
    asset::{CanonicalContract, Contract, ResponseStatus},
    protocols::{
        secretswap::{
            AssetInfo, SecretSwapPairInfo, SecretSwapPairQueryMsg, SecretSwapPoolResponse,
        },
        siennaswap::{SiennaDexTokenType, SiennaSwapExchangeQueryMsg, SiennaSwapPairInfo},
    },
    querier::query_token_info,
    scrt::{
        debug_print, to_binary, Api, CanonicalAddr, Env, Extern, HandleResponse, HumanAddr,
        InitResponse, Querier, QueryRequest, QueryResult, StdError, StdResult, Storage, Uint128,
        WasmQuery, BLOCK_SIZE,
    },
    secret_toolkit::utils::{pad_handle_result, pad_query_result},
    storage::traits::SingletonStorable,
};
use std::cmp::min;

/// state of the auction
#[derive(Serialize, Deserialize)]
pub struct State {
    pub owner: CanonicalAddr,
    pub oracle0: CanonicalContract,
    pub oracle1: CanonicalContract,
    pub factory: CanonicalContract,
    pub lp_token: CanonicalContract,
    pub dex: Dex,
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
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let oracle0: CanonicalContract = CanonicalContract {
        address: deps
            .api
            .canonical_address(&HumanAddr(msg.oracle0.address))?,
        code_hash: msg.oracle0.code_hash,
    };

    let oracle1: CanonicalContract = CanonicalContract {
        address: deps
            .api
            .canonical_address(&HumanAddr(msg.oracle1.address))?,
        code_hash: msg.oracle1.code_hash,
    };

    let factory: CanonicalContract = CanonicalContract {
        address: deps
            .api
            .canonical_address(&HumanAddr(msg.factory.address.clone()))?,
        code_hash: msg.factory.code_hash.clone(),
    };

    let lp_token;
    let mut token0: Contract = Contract {
        address: "a".to_string(),
        code_hash: "b".to_string(),
    };
    let mut token1: Contract = Contract {
        address: "a".to_string(),
        code_hash: "b".to_string(),
    };

    match msg.dex {
        Dex::SecretSwap => {
            let pair_info: SecretSwapPairInfo =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: HumanAddr::from(msg.factory.address),
                    callback_code_hash: msg.factory.code_hash,
                    msg: to_binary(&SecretSwapPairQueryMsg::Pair {})?,
                }))?;
            lp_token = CanonicalContract {
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
        }
        Dex::SiennaSwap => {
            let pair_info: SiennaSwapPairInfo =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: HumanAddr::from(msg.factory.address.clone()),
                    callback_code_hash: msg.factory.code_hash.clone(),
                    msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
                }))?;
            lp_token = CanonicalContract {
                address: deps
                    .api
                    .canonical_address(&HumanAddr::from(pair_info.liquidity_token.address))?,
                code_hash: pair_info.liquidity_token.code_hash,
            };
            if let SiennaDexTokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } = &pair_info.pair[0]
            {
                token0.address = contract_addr.to_string();
                token0.code_hash = token_code_hash.to_string();
            } else {
                return Err(StdError::generic_err(
                    "Could not resolve SiennaSwap token 1 info.",
                ));
            }
            if let SiennaDexTokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } = &pair_info.pair[1]
            {
                token1.address = contract_addr.to_string();
                token1.code_hash = token_code_hash.to_string();
            } else {
                return Err(StdError::generic_err(
                    "Could not resolve SiennaSwap token 2 info.",
                ));
            }
        }
    }

    let token0_decimals = query_token_info(&token0, &deps.querier)?
        .token_info
        .decimals;
    let token1_decimals = query_token_info(&token1, &deps.querier)?
        .token_info
        .decimals;

    let state: State = State {
        owner: deps.api.canonical_address(&HumanAddr(msg.owner))?,
        oracle0,
        oracle1,
        factory,
        lp_token,
        dex: msg.dex,
        token0_decimals,
        token1_decimals,
    };

    state.save_json(&mut deps.storage)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

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
                oracle0,
                oracle1,
                factory,
                dex,
            } => try_update_config(deps, env, owner, oracle0, oracle1, factory, dex),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<String>,
    oracle0: Option<Contract>,
    oracle1: Option<Contract>,
    factory: Option<Contract>,
    dex: Option<Dex>,
) -> StdResult<HandleResponse> {
    let mut state: State = State::new_json(&deps.storage)?;

    if deps.api.canonical_address(&env.message.sender)? != state.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    if let Some(owner) = owner {
        state.owner = deps.api.canonical_address(&HumanAddr(owner))?;
    }

    if let Some(oracle0) = oracle0 {
        let oracle0 = CanonicalContract {
            address: deps.api.canonical_address(&HumanAddr(oracle0.address))?,
            code_hash: oracle0.code_hash,
        };
        state.oracle0 = oracle0;
    }

    if let Some(oracle1) = oracle1 {
        let oracle1 = CanonicalContract {
            address: deps.api.canonical_address(&HumanAddr(oracle1.address))?,
            code_hash: oracle1.code_hash,
        };
        state.oracle1 = oracle1;
    }

    if let Some(factory) = factory {
        let factory = CanonicalContract {
            address: deps.api.canonical_address(&HumanAddr(factory.address))?,
            code_hash: factory.code_hash,
        };
        state.factory = factory;
    }

    state.dex = dex.unwrap_or(state.dex);

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
        oracle1: state.oracle0.as_human(&deps.api)?,
        oracle2: state.oracle1.as_human(&deps.api)?,
        factory: state.factory.as_human(&deps.api)?,
        dex: state.dex,
    })
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<PriceResponse> {
    let state: State = State::new_json(&deps.storage)?;

    let price0: PriceResponse = query_price(&state.oracle0.as_human(&deps.api)?, &deps.querier)?;

    let price1: PriceResponse = query_price(&state.oracle1.as_human(&deps.api)?, &deps.querier)?;

    let reserve0;
    let reserve1;

    match state.dex {
        Dex::SecretSwap => {
            let pair_info: SecretSwapPoolResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: deps.api.human_address(&state.factory.address)?,
                    callback_code_hash: state.factory.code_hash,
                    msg: to_binary(&SecretSwapPairQueryMsg::Pool {})?,
                }))?;
            reserve0 = pair_info.assets[0].amount;
            reserve1 = pair_info.assets[1].amount;
        }
        Dex::SiennaSwap => {
            let pair_info: SiennaSwapPairInfo =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: deps.api.human_address(&state.factory.address)?,
                    callback_code_hash: state.factory.code_hash,
                    msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
                }))?;
            reserve0 = pair_info.amount_0;
            reserve1 = pair_info.amount_1;
        }
    }

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
