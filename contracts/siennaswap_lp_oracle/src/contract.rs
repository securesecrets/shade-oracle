use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::querier::{query_price, query_token_info},
    common::{CanonicalContract, Contract},
    common::{CommonOracleConfig, HandleMsg, HandleStatusAnswer, PriceResponse, QueryMsg},
    lp::{
        get_fair_lp_token_price,
        siennaswap::{ConfigResponse, InitMsg},
        FairLpPriceInfo,
    },
    protocols::siennaswap::{
        SiennaDexTokenType, SiennaSwapExchangeQueryMsg, SiennaSwapPairInfoResponse,
    },
    router::querier::query_oracle,
    scrt::{
        secret_toolkit::utils::{pad_handle_result, pad_query_result},
        to_binary, Api, CanonicalAddr, Env, Extern, HandleResponse, HumanAddr, InitResponse,
        Querier, QueryRequest, QueryResult, StdError, StdResult, Storage, Uint128, WasmQuery,
        BLOCK_SIZE,
    },
    storage::Item,
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
    pub enabled: bool,
}

const STATE: Item<State> = Item::new("state");
const COMMON: Item<CommonOracleConfig> = Item::new("common");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let router: CanonicalContract = CanonicalContract {
        address: deps.api.canonical_address(&msg.router.address)?,
        code_hash: msg.router.code_hash,
    };

    let factory: CanonicalContract = CanonicalContract {
        address: deps.api.canonical_address(&msg.factory.address.clone())?,
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

    let pair_info_response: SiennaSwapPairInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: HumanAddr::from(msg.factory.address.clone()),
            callback_code_hash: msg.factory.code_hash.clone(),
            msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
        }))?;
    let pair_info = pair_info_response.pair_info;
    let lp_token = CanonicalContract {
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

    let token0_decimals = query_token_info(&token0, &deps.querier)?
        .token_info
        .decimals;
    let token1_decimals = query_token_info(&token1, &deps.querier)?
        .token_info
        .decimals;

    let state: State = State {
        owner: deps.api.canonical_address(&msg.owner)?,
        symbol_0: msg.symbol_0,
        symbol_1: msg.symbol_1,
        router,
        factory,
        lp_token,
        token0_decimals,
        token1_decimals,
        enabled: true,
    };

    state.save_json(&mut deps.storage)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    pad_handle_result(
        match msg {
            HandleMsg::SetStatus { enabled } => try_update_status(deps, enabled),
        },
        BLOCK_SIZE,
    )
}

fn try_update_status<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    new_status: bool,
) -> StdResult<HandleResponse> {
    let mut state: State = State::new_json(&deps.storage)?;

    state.enabled = new_status;

    state.save_json(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleStatusAnswer { new_status })?),
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
        owner: deps.api.human_address(&state.owner)?,
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

    let pair_info_response: SiennaSwapPairInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&state.factory.address)?,
            callback_code_hash: state.factory.code_hash,
            msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
        }))?;
    let pair_info = pair_info_response.pair_info;
    let reserve0 = pair_info.amount_0;
    let reserve1 = pair_info.amount_1;

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
