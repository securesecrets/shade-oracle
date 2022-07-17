use cosmwasm_std::{Uint128, entry_point};
use cosmwasm_std::{
    to_binary, MessageInfo, Binary, Env, Deps, Response, Addr,  DepsMut,
    QueryRequest, StdError, StdResult, QueryResponse, WasmQuery,
};
use shade_oracles::{
    pad_handle_result, pad_query_result, Contract, ResponseStatus, BLOCK_SIZE,
    interfaces::band::ReferenceData,
    common::querier::{query_prices, query_token_info, verify_admin},
    common::{is_disabled, HandleAnswer, ExecuteMsg, OraclePrice, OracleQuery},
    common::{throw_unsupported_symbol_error},
    interfaces::lp::{
        get_fair_lp_token_price,
        siennaswap::{Config, InstantiateMsg, PairData},
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

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let mut token0 = Contract::new(&Addr::unchecked("a"), &"b".to_string());
    let mut token1 = token0.clone();

    let pair_info_response: SiennaSwapPairInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.exchange.address.to_string(),
            code_hash: msg.exchange.code_hash.clone(),
            msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
        }))?;
    let pair_info = pair_info_response.pair_info;
    let lp_token = pair_info.liquidity_token;
    if let SiennaDexTokenType::CustomToken {
        contract_addr,
        token_code_hash,
    } = &pair_info.pair.token_0
    {
        token0.address = contract_addr.clone();
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
        token1.address = contract_addr.clone();
        token1.code_hash = token_code_hash.to_string();
    } else {
        return Err(StdError::generic_err(
            "Could not resolve SiennaSwap token 2 info.",
        ));
    }

    let token0_decimals = query_token_info(&token0, &deps.querier)?
        .decimals;
    let token1_decimals = query_token_info(&token1, &deps.querier)?
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

    CONFIG.save(deps.storage, &config)?;
    PAIR.save(deps.storage, &pair)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    verify_admin(&config.router, deps.as_ref(), info.sender.clone())?;

    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateConfig { enabled } => try_update_config(deps, enabled),
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: OracleQuery) -> StdResult<QueryResponse> {
    pad_query_result(
        match msg {
            OracleQuery::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
            OracleQuery::GetPrice { key } => try_query_price(deps, key),
            OracleQuery::GetPrices { .. } => {
                Err(StdError::generic_err("GetPrices method not supported."))
            }
        },
        BLOCK_SIZE,
    )
}

fn try_query_price(
    deps: Deps,
    key: String,
) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    is_disabled(config.enabled)?;
    let pair = PAIR.load(deps.storage)?;

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
            contract_addr: config.exchange.address.to_string(),
            code_hash: config.exchange.code_hash,
            msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
        }))?;
    let pair_info = pair_info_response.pair_info;
    let reserve0 = pair_info.amount_0;
    let reserve1 = pair_info.amount_1;

    let lp_token_info = query_token_info(&pair.lp_token, &deps.querier)?;

    let total_supply = lp_token_info.total_supply.unwrap();
    let lp_token_decimals = lp_token_info.decimals;

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
