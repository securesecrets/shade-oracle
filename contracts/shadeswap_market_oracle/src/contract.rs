use shade_oracles::{
    common::{
        normalize_price,
        querier::{query_token_info, verify_admin, query_band_price, query_price},
        BLOCK_SIZE, Contract, 
        ResponseStatus, 
        OraclePrice, HandleAnswer, QueryMsg
    },
    protocols::shadeswap::{
        TokenType, 
        TokenAmount,
        ShadeSwapQueryMsg, 
        PairInfoResponse,
        EstimatedPriceResponse,
    },
    storage::Item,
    band::ReferenceData,
    shadeswap_market_oracle::{
        Config, InitMsg,
        HandleMsg
    },
};
use cosmwasm_std::{
    to_binary, Api, Env, 
    Extern, HandleResponse, 
    HumanAddr, InitResponse,
    Querier, QueryResult, 
    StdError, StdResult, Storage, 
};
use cosmwasm_math_compat::Uint128;
use secret_toolkit::{
    utils::{Query, pad_handle_result, pad_query_result},
    snip20::TokenInfo,
};

const CONFIG: Item<Config> = Item::new("config");

const PRIMARY_TOKEN: Item<Contract> = Item::new("primary_token");
const PRIMARY_INFO: Item<TokenInfo> = Item::new("primary_info");
const BASE_INFO: Item<TokenInfo> = Item::new("base_info");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let pair_info_response: PairInfoResponse =
        ShadeSwapQueryMsg::GetPairInfo.query(
            &deps.querier,
            msg.pair.code_hash.clone(),
            msg.pair.address.clone(),
        )?;

    let tokens: [Contract; 2] = vec![
        pair_info_response.pair.0,
        pair_info_response.pair.1,
    ].iter()
        .filter_map(|t| match t {
            TokenType::CustomToken {
                contract_addr, token_code_hash,
            } => Some(Contract {
                address: HumanAddr(contract_addr.to_string()),
                code_hash: token_code_hash.to_string(),
            }),
            _ => None,
        })
        .collect::<Vec<Contract>>()
        .try_into()
        .ok()
        .unwrap();

    let token_infos: [TokenInfo; 2] = tokens
        .iter()
        .map(|t| query_token_info(t, &deps.querier)
                    .ok()
                    .unwrap()
                    .token_info
        )
        .collect::<Vec<TokenInfo>>()
        .try_into()
        .ok()
        .unwrap();

    let primary_i = match token_infos.iter().position(|t| t.symbol == msg.symbol) {
        Some(i) => i,
        None => {
            return Err(StdError::generic_err(format!("Neither token matches {}", msg.symbol)));
        }
    };

    let base_i = token_infos.iter().position(|t| t.symbol != msg.symbol).unwrap();

    let config = Config {
        router: msg.router,
        pair: msg.pair,
        symbol: msg.symbol.clone(),
        base_peg: match msg.base_peg {
            Some(p) => p,
            None => token_infos[base_i].symbol.clone(),
        },
        enabled: true,
    };

    if let Err(e) = query_band_price(&config.router, &deps.querier, config.base_peg.clone()) {
        return Err(StdError::generic_err(format!(
                    "Failed to query base_peg {} from router {}; {}", 
                    config.base_peg, config.router.address, e)));
    };

    CONFIG.save(&mut deps.storage, &config)?;
    PRIMARY_TOKEN.save(&mut deps.storage, &tokens[primary_i].clone())?;
    PRIMARY_INFO.save(&mut deps.storage, &token_infos[primary_i])?;
    BASE_INFO.save(&mut deps.storage, &token_infos[base_i])?;

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
                router,
                enabled,
            } => try_update_config(deps, &env, router, enabled),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    router: Option<Contract>,
    enabled: Option<bool>,
) -> StdResult<HandleResponse> {

    let mut config = CONFIG.load(&deps.storage)?;

    verify_admin(&config.router, &deps.querier, env.message.sender.clone())?;

    if let Some(router) = router {
        config.router = router;
    }

    if let Some(enabled) = enabled {
        config.enabled = enabled;
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
            QueryMsg::GetConfig {} => to_binary(&CONFIG.load(&deps.storage)?),
            QueryMsg::GetPrice { key } => to_binary(&try_query_price(deps, key)?),
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

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: String,
) -> StdResult<OraclePrice> {
    let config = CONFIG.load(&deps.storage)?;

    let primary_token: Contract = PRIMARY_TOKEN.load(&deps.storage)?;

    let primary_info = PRIMARY_INFO.load(&deps.storage)?;

    // Simulate trade 1 primary -> 1 base
    let sim: EstimatedPriceResponse = ShadeSwapQueryMsg::GetEstimatedPrice {
        offer: TokenAmount {
            amount: Uint128::from(10u128.pow(primary_info.decimals.into())),
            token: TokenType::CustomToken {
                contract_addr: primary_token.address,
                token_code_hash: primary_token.code_hash,
            },
        }
    }.query(
        &deps.querier,
        config.pair.code_hash.clone(),
        config.pair.address.clone(),
    )?;

    // Normalize to 'rate * 10^18'
    let base_info = BASE_INFO.load(&deps.storage)?;
    let exchange_rate = normalize_price(sim.estimated_price, base_info.decimals);

    // Query router for base_peg/USD
    let base_usd_price = if config.only_band {
        query_band_price(&config.router, &deps.querier, config.base_peg)?
    } else {
        query_price(&config.router, &deps.querier, config.base_peg)?
    };

    // Translate price to primary/USD
    let price = base_usd_price.data.rate.multiply_ratio(exchange_rate, 10u128.pow(18));

    Ok(OraclePrice::new(key,
        ReferenceData {
            rate: price,
            last_updated_base: base_usd_price.data.last_updated_base,
            last_updated_quote: base_usd_price.data.last_updated_quote,
        }
    ))
}
