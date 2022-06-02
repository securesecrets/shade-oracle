use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::{
        querier::{query_price, query_token_info},
        BLOCK_SIZE, Contract, 
        ResponseStatus, 
        throw_unsupported_symbol_error,
        HandleMsg, HandleStatusAnswer, 
        OraclePrice, QueryMsg
    },
    protocols::siennaswap::{
        SiennaDexTokenType, 
        SiennaSwapExchangeQueryMsg, 
        SiennaSwapPairInfoResponse,
    },
    router::querier::query_oracle,
    storage::Item,
    band::ReferenceData,
    siennaswap_market_oracle::{
        Config, InitMsg,
        HandleMsg, HandleAnswer,
        QueryMsg,
    },
};
use cosmwasm_std::{
    to_binary, Api, Env, 
    Extern, HandleResponse, 
    HumanAddr, InitResponse,
    Querier, QueryRequest, QueryResult, 
    StdError, StdResult, Storage, 
    Uint128, WasmQuery, Binary,
};
use secret_toolkit::utils::{
    pad_handle_result, pad_query_result,
    utils::Query,
    snip20::TokenInfo,
};

const CONFIG: Item<Config> = Item::new("config");
const TOKEN_INFOS: Item<[TokenInfo; 2]> = Item::new("token_infos");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let pair_info_response: SiennaSwapPairInfoResponse =
        SiennaSwapExchangeQueryMsg::PairInfo.query(
            msg.pair.address.clone(),
            msg.pair.code_hash.clone(),
        )?;
    let pair_info = pair_info_response.pair_info;

    let token0 = match pair_info.pair[0] {
        SiennaDexTokenType::CustomToken {
            contract_addr, token_code_hash,
        } => Contract {
                address: HumanAddr(contract_addr),
                code_hash: token_code_hash,
            },
        _ => {
            return Err(StdError::generic_err(
                    "Failed to get token 0 contract from pair"));
        }
    };

    let token1 = match pair_info.pair[1] {
        SiennaDexTokenType::CustomToken {
            contract_addr, token_code_hash,
        } => Contract {
            address: contract_addr,
            code_hash: token_code_hash,
        },
        _ => {
            return Err(StdError::generic_err(
                    "Failed to get token 1 contract from pair"));
        }
    };

    let token_infos = vec![
        query_token_info(&token0, &deps.querier)?.token_info,
        query_token_info(&token1, &deps.querier)?.token_info,
    ];

    let config = Config {
        admins: match msg.admins {
            Some(admins) => {
                if !admins.contains(&env.message.sender) {
                    admins.push(env.message.sender);
                }
                admins
            },
            None => vec![env.message.sender],
        },
        router: msg.router,
        pair: msg.pair,
        symbol: msg.symbol,
        base_peg: match msg.base_peg {
            Some(p) => p,
            None => {
                if token_infos[0].symbol == msg.symbol {
                    token_infos[1].symbol
                }
                else if token_infos[1].symbol == msg.symbol {
                    token_infos[0].symbol
                }
                else {
                    return Err(StdError::generic_err(format!(
                        "Neither asset aligns with the {}", msg.symbol)));
                }
            }
        }
    };

    /*TODO
     * Query for the base peg from router to verify
     */

    CONFIG.save(&mut deps.storage, &config)?;
    TOKEN_INFOS.save(&mut deps.storage, &token_infos)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    pad_handle_result(
        match msg {
            HandleMsg::UpdateConfig { 
                admins, router,
            } => try_update_config(deps, &env, admins, router),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    admins: Option<Vec<HumanAddr>>,
    router: Option<Contract>,
) -> StdResult<HandleResponse> {

    let mut config = CONFIG.load(&deps.storage)?;

    if !config.admins.contains(&env.message.sender) {
        return Err(StdError::unauthorized());
    }

    if let Some(admins) = admins {
        if !admins.is_empty() {
            config.admins = admins;
        }
    }

    if let Some(router) = router {
        config.router = router;
    }

    CONFIG.save(&mut deps.storage, &config)?;

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
            QueryMsg::GetPrice { key } => try_query_price(deps, key),
            QueryMsg::GetPrices { keys } => {

                let mut prices = vec![];
                for key in keys {
                    prices.push(try_query_price(deps, key)?);
                }
                to_binary(&prices)
            },
        },
        BLOCK_SIZE,
    )
}

fn try_query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Config> {
    CONFIG.load(&deps.storage)
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: String,
) -> StdResult<OraclePrice> {
    /*
    let state = STATE.load(&deps.storage)?;

    if key != state.supported_key {
        return Err(throw_unsupported_symbol_error(key));
    }
    
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
    */
    OraclePrice::new(key, ReferenceData {
        rate: Uint128::zero(),
        last_updated_base: 0,
        last_updated_quote: 0,
    })
}
