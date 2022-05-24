use shade_oracles::{
    scrt::{
        serde::{Deserialize, Serialize},
        to_binary, Api, Env, Extern, HandleResponse, HumanAddr, InitResponse,
        Querier, QueryRequest, QueryResult, StdError, StdResult, Storage, Uint128, WasmQuery,
        BLOCK_SIZE,
        secret_toolkit::utils::{pad_query_result},
    },
    common::{Contract, CanonicalContract, querier::{query_token_info, query_price}, PriceResponse, QueryMsg, CommonOracleConfig, HandleMsg},
    lp::{
        get_lp_token_spot_price,
        siennaswap::{
            ConfigResponse, InitMsg, SiennaDexTokenType,
            SiennaSwapExchangeQueryMsg, SiennaSwapQueryResponse,
        },
        FairLpPriceInfo,
    },
    storage::Item,
    router::querier::query_oracle,
};
use std::cmp::min;

#[derive(Serialize, Deserialize)]
pub struct State {
    pub symbol_0: String,
    pub symbol_1: String,
    pub router: CanonicalContract,
    pub factory: CanonicalContract,
    pub lp_token: CanonicalContract,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
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
        address: deps
            .api
            .canonical_address(&msg.factory.address.clone())?,
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

    let pair_info_response: SiennaSwapQueryResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: HumanAddr::from(msg.factory.address.clone()),
            callback_code_hash: msg.factory.code_hash.clone(),
            msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
        }))?;

    let lp_token: CanonicalContract;

    match pair_info_response {
        SiennaSwapQueryResponse::PairInfo {
            liquidity_token,
            factory: _,
            pair,
            amount_0: _,
            amount_1: _,
            total_liquidity: _,
            contract_version: _,
        } => {
            lp_token = CanonicalContract {
                address: deps
                    .api
                    .canonical_address(&HumanAddr::from(liquidity_token.address))?,
                code_hash: liquidity_token.code_hash,
            };
            if let SiennaDexTokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } = &pair.token_0
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
            } = &pair.token_1
            {
                token1.address = HumanAddr(contract_addr.to_string());
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
        symbol_0: msg.symbol_0,
        symbol_1: msg.symbol_1,
        router,
        factory,
        lp_token,
        token0_decimals,
        token1_decimals,
    };

    let common: CommonOracleConfig = CommonOracleConfig { owner: msg.owner, enabled: true };

    COMMON.save(&mut deps.storage, &common)?;
    STATE.save(&mut deps.storage, &state)?;

    Ok(InitResponse::default())
}

/* CONFIG UPDATE NEEDS TO BE FIXED */
pub fn handle<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: HandleMsg,
) -> StdResult<HandleResponse> {

    Err(StdError::generic_err("Handle messages are disabled."))
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => to_binary(&try_query_config(deps)?),
            QueryMsg::GetPrice { .. } => to_binary(&try_query_price(deps)?),
            QueryMsg::GetPrices { symbols } => Err(StdError::generic_err("Unsupported method.")),
        },
        BLOCK_SIZE,
    )
}

fn try_query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let state: State = STATE.load(&deps.storage)?;
    let common: CommonOracleConfig = COMMON.load(&deps.storage)?;

    Ok(ConfigResponse {
        owner: common.owner,
        symbol_0: state.symbol_0,
        symbol_1: state.symbol_1,
        router: state.router.as_human(&deps.api)?,
        factory: state.factory.as_human(&deps.api)?,
        enabled: common.enabled,
    })
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<PriceResponse> {
    let state: State = STATE.load(&deps.storage)?;

    // NEED TO UPDATE THIS WHEN ORACLE ROUTER IS DONE

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

    let price0: PriceResponse = query_price(&oracle0, &deps.querier, state.symbol_0)?;

    let price1: PriceResponse = query_price(&oracle1, &deps.querier, state.symbol_1)?;

    let pair_info_response: SiennaSwapQueryResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&state.factory.address)?,
            callback_code_hash: state.factory.code_hash,
            msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
        }))?;

    let reserve0;
    let reserve1;

    match pair_info_response {
        SiennaSwapQueryResponse::PairInfo {
            liquidity_token: _,
            factory: _,
            pair: _,
            amount_0,
            amount_1,
            total_liquidity: _,
            contract_version: _,
        } => {
            reserve0 = amount_0;
            reserve1 = amount_1;
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

    let price = get_lp_token_spot_price(a, b, total_supply.u128(), lp_token_decimals);

    let response = PriceResponse {
        rate: Uint128(price.unwrap()),
        last_updated_base: min(price0.last_updated_base, price1.last_updated_base),
        last_updated_quote: min(price0.last_updated_quote, price1.last_updated_quote),
    };
    Ok(response)
}
