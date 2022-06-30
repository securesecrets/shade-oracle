use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
    QueryRequest, QueryResult, StdError, StdResult, Storage, WasmQuery,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use shade_oracles::{
    band::ReferenceData,
    common::querier::{query_prices, query_token_info, verify_admin},
    common::{is_disabled, HandleAnswer, HandleMsg, OraclePrice, QueryMsg},
    common::{throw_unsupported_symbol_error, Contract, ResponseStatus, BLOCK_SIZE},
    lp::{
        get_fair_lp_token_price,
        siennaswap::{Config, InitMsg, PairData},
        FairLpPriceInfo,
    },
    protocols::siennaswap::{
        SiennaDexTokenType, SiennaSwapExchangeQueryMsg, SiennaSwapPairInfoResponse,
    },
    storage::Item,
};
use std::cmp::min;

const PAIR: Item<PairData> = Item::new("pair");
const CONFIG: Item<Config> = Item::new("config");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let mut token0 = Contract::new("a".to_string(), "b".to_string());
    let mut token1 = token0.clone();

    let pair_info_response: SiennaSwapPairInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.exchange.address.clone(),
            callback_code_hash: msg.exchange.code_hash.clone(),
            msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
        }))?;
    let pair_info = pair_info_response.pair_info;
    let lp_token = pair_info.liquidity_token;
    if let SiennaDexTokenType::CustomToken {
        contract_addr,
        token_code_hash,
    } = &pair_info.pair.token_0
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
    } = &pair_info.pair.token_1
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

    let config = Config {
        supported_key: msg.supported_key,
        symbol_0: msg.symbol_0,
        symbol_1: msg.symbol_1,
        router: msg.router,
        exchange: msg.exchange,
        enabled: true,
    };

    let pair = PairData {
        lp_token,
        token0_decimals,
        token1_decimals,
    };

    CONFIG.save(&mut deps.storage, &config)?;
    PAIR.save(&mut deps.storage, &pair)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    pad_handle_result(
        match msg {
            HandleMsg::UpdateConfig { enabled } => try_update_config(deps, &env, enabled),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    enabled: bool,
) -> StdResult<HandleResponse> {
    let config = CONFIG.load(&deps.storage)?;
    verify_admin(&config.router, &deps.querier, env.message.sender.clone())?;
    CONFIG.update(&mut deps.storage, |mut config| -> StdResult<_> {
        config.enabled = enabled;
        Ok(config)
    })?;
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
            QueryMsg::GetConfig {} => to_binary(&CONFIG.load(&deps.storage)?),
            QueryMsg::GetPrice { key } => try_query_price(deps, key),
            QueryMsg::GetPrices { .. } => {
                Err(StdError::generic_err("GetPrices method not supported."))
            }
        },
        BLOCK_SIZE,
    )
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: String,
) -> StdResult<Binary> {
    let config = CONFIG.load(&deps.storage)?;
    is_disabled(config.enabled)?;
    let pair = PAIR.load(&deps.storage)?;

    if key != config.supported_key {
        return Err(throw_unsupported_symbol_error(key));
    }

    let prices = query_prices(
        &config.router,
        &deps.querier,
        vec![config.symbol_0, config.symbol_1],
    )?;
    let (price0, price1) = (prices[0].clone(), prices[1].clone());

    let pair_info_response: SiennaSwapPairInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.exchange.address.clone(),
            callback_code_hash: config.exchange.code_hash,
            msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
        }))?;
    let pair_info = pair_info_response.pair_info;
    let reserve0 = pair_info.amount_0;
    let reserve1 = pair_info.amount_1;

    let lp_token_info = query_token_info(&pair.lp_token, &deps.querier)?;

    let total_supply = lp_token_info.token_info.total_supply.unwrap();
    let lp_token_decimals = lp_token_info.token_info.decimals;

    let a = FairLpPriceInfo {
        reserve: reserve0.u128(),
        price: price0.data.rate.u128(),
        decimals: pair.token0_decimals,
    };

    let b = FairLpPriceInfo {
        reserve: reserve1.u128(),
        price: price1.data.rate.u128(),
        decimals: pair.token1_decimals,
    };

    let price = get_fair_lp_token_price(a, b, total_supply.u128(), lp_token_decimals);

    let data = ReferenceData {
        rate: Uint128::from(u128::from_be_bytes(price.unwrap().to_be_bytes())),
        last_updated_base: min(price0.data.last_updated_base, price1.data.last_updated_base),
        last_updated_quote: min(
            price0.data.last_updated_quote,
            price1.data.last_updated_quote,
        ),
    };
    to_binary(&OraclePrice::new(key, data))
}
