use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::querier::{query_price, query_token_info},
    common::{BLOCK_SIZE, CanonicalContract, Contract, ResponseStatus},
    common::{CommonOracleConfig, HandleMsg, HandleStatusAnswer, OraclePrice, QueryMsg},
    lp::{
        get_fair_lp_token_price,
        siennaswap::{ConfigResponse, InitMsg},
        FairLpPriceInfo,
    },
    protocols::siennaswap::{
        SiennaDexTokenType, SiennaSwapExchangeQueryMsg, SiennaSwapPairInfoResponse,
    },
    router::{querier::query_oracle},
    storage::Item, band::ReferenceData,
};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, QueryRequest, QueryResult, StdError, StdResult, Storage, Uint128, WasmQuery, Binary,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use std::cmp::min;

#[derive(Serialize, Deserialize)]
pub struct State {
    pub supported_symbol: String,
    pub symbol_0: String,
    pub symbol_1: String,
    pub router: CanonicalContract,
    pub factory: CanonicalContract,
    pub lp_token: CanonicalContract,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
}

const STATE: Item<State> = Item::new("state");
const CONFIG: Item<CommonOracleConfig> = Item::new("common");

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
        address: deps.api.canonical_address(&msg.factory.address)?,
        code_hash: msg.factory.code_hash.clone(),
    };

    let mut token0: Contract = Contract {
        address: HumanAddr("a".to_string()),
        code_hash: "b".to_string(),
    };
    let mut token1: Contract = Contract {
        address: HumanAddr("a".to_string()),
        code_hash: "b".to_string(),
    };

    let pair_info_response: SiennaSwapPairInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.factory.address.clone(),
            callback_code_hash: msg.factory.code_hash.clone(),
            msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
        }))?;
    let pair_info = pair_info_response.pair_info;
    let lp_token = CanonicalContract {
        address: deps
            .api
            .canonical_address(&pair_info.liquidity_token.address)?,
        code_hash: pair_info.liquidity_token.code_hash,
    };
    if let SiennaDexTokenType::CustomToken {
        contract_addr,
        token_code_hash,
    } = &pair_info.pair[0]
    {
        token0.address = HumanAddr(contract_addr.to_string());
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
        token1.address = HumanAddr(contract_addr.to_string());
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
        supported_symbol: msg.supported_symbol,
        symbol_0: msg.symbol_0,
        symbol_1: msg.symbol_1,
        router,
        factory,
        lp_token,
        token0_decimals,
        token1_decimals,
    };

    let config = CommonOracleConfig {
        owner: msg.owner,
        enabled: true,
    };

    STATE.save(&mut deps.storage, &state)?;
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
            HandleMsg::SetStatus { enabled } => try_update_status(deps, &env, enabled),
        },
        BLOCK_SIZE,
    )
}

fn try_update_status<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    enabled: bool,
) -> StdResult<HandleResponse> {
    CONFIG.load(&deps.storage)?.is_owner(env)?;
    let new_config = CONFIG.update(&mut deps.storage, |mut config| -> StdResult<_> {
        config.enabled = enabled;
        Ok(config)
    })?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleStatusAnswer { status: ResponseStatus::Success, enabled: new_config.enabled, })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => to_binary(&try_query_config(deps)?),
            QueryMsg::GetPrice { symbol } => to_binary(&try_query_price(deps,symbol)?),
            QueryMsg::GetPrices { .. } => Err(StdError::generic_err("GetPrices method not supported.")),
        },
        BLOCK_SIZE,
    )
}

fn try_query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let state = STATE.load(&deps.storage)?;
    let config = CONFIG.load(&deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        symbol_0: state.symbol_0,
        symbol_1: state.symbol_1,
        router: state.router.as_human(&deps.api)?,
        factory: state.factory.as_human(&deps.api)?,
        supported_symbol: state.supported_symbol,
        enabled: config.enabled,
    })
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
) -> StdResult<Binary> {
    let state = STATE.load(&deps.storage)?;

    let oracle0 = query_oracle(
        &state.router.as_human(&deps.api)?,
        &deps.querier,
        state.symbol_0.clone(),
    )?;
    let oracle1 = query_oracle(
        &state.router.as_human(&deps.api)?,
        &deps.querier,
        state.symbol_1.clone(),
    )?;

    let price0 = query_price(&oracle0, &deps.querier, state.symbol_0)?;

    let price1 = query_price(&oracle1, &deps.querier, state.symbol_1)?;

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
        price: price0.price.rate.u128(),
        decimals: state.token0_decimals,
    };

    let b = FairLpPriceInfo {
        reserve: reserve1.u128(),
        price: price1.price.rate.u128(),
        decimals: state.token1_decimals,
    };

    let price = get_fair_lp_token_price(a, b, total_supply.u128(), lp_token_decimals);

    let data = ReferenceData {
        rate: Uint128(price.unwrap()),
        last_updated_base: min(price0.price.last_updated_base, price1.price.last_updated_base),
        last_updated_quote: min(price0.price.last_updated_quote, price1.price.last_updated_quote),
    };
    to_binary(&OraclePrice::new(symbol, data))
}
