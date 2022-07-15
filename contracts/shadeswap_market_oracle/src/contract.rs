use cosmwasm_std::Uint128;
use cosmwasm_std::{
    to_binary, Api, Env, Deps, Response, Addr,  Querier, 
   StdError, StdResult, Storage,
};
use secret_toolkit::{
    snip20::TokenInfo,
    utils::{pad_handle_result, pad_query_result, Query},
};
use shade_oracles::{
    band::ReferenceData,
    common::{
        normalize_price,
        querier::{query_band_price, query_price, query_token_info, verify_admin},
        Contract, HandleAnswer, OraclePrice, QueryMsg, ResponseStatus, BLOCK_SIZE,
    },
    protocols::shadeswap::{
        EstimatedPriceResponse, PairInfoResponse, ShadeSwapQueryMsg, TokenAmount, TokenType,
    },
    shadeswap_market_oracle::{Config, ExecuteMsg, InstantiateMsg},
    storage::Item,
};

const CONFIG: Item<Config> = Item::new("config");

const PRIMARY_TOKEN: Item<Contract> = Item::new("primary_token");
const PRIMARY_INFO: Item<TokenInfo> = Item::new("primary_info");
const BASE_INFO: Item<TokenInfo> = Item::new("base_info");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let pair_info_response: PairInfoResponse = ShadeSwapQueryMsg::GetPairInfo.query(
        &deps.querier,
        msg.pair.code_hash.clone(),
        msg.pair.address.clone(),
    )?;

    let tokens: [Contract; 2] = vec![pair_info_response.pair.0, pair_info_response.pair.1]
        .iter()
        .filter_map(|t| match t {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => Some(Contract {
                address: *contract_addr,
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
        .map(|t| query_token_info(t, &deps.querier).ok().unwrap().token_info)
        .collect::<Vec<TokenInfo>>()
        .try_into()
        .ok()
        .unwrap();

    let primary_i = match token_infos.iter().position(|t| t.symbol == msg.symbol) {
        Some(i) => i,
        None => {
            return Err(StdError::generic_err(format!(
                "Neither token matches {}",
                msg.symbol
            )));
        }
    };

    let base_i = token_infos
        .iter()
        .position(|t| t.symbol != msg.symbol)
        .unwrap();

    let config = Config {
        router: msg.router,
        pair: msg.pair,
        symbol: msg.symbol.clone(),
        base_peg: match msg.base_peg {
            Some(p) => p,
            None => token_infos[base_i].symbol.clone(),
        },
        enabled: true,
        only_band: msg.only_band,
    };

    if let Err(e) = query_band_price(&config.router, &deps.querier, config.base_peg.clone()) {
        return Err(StdError::generic_err(format!(
            "Failed to query base_peg {} from router {}; {}",
            config.base_peg, config.router.address, e
        )));
    };

    CONFIG.save(deps.storage, &config)?;
    PRIMARY_TOKEN.save(deps.storage, &tokens[primary_i].clone())?;
    PRIMARY_INFO.save(deps.storage, &token_infos[primary_i])?;
    BASE_INFO.save(deps.storage, &token_infos[base_i])?;

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
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateConfig {
                router,
                enabled,
                only_band,
            } => try_update_config(deps, &env, router, enabled, only_band),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config(
    deps: DepsMut,
    env: &Env,
    router: Option<Contract>,
    enabled: Option<bool>,
    only_band: Option<bool>,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    verify_admin(&config.router, &deps.querier, env.message.sender.clone())?;

    if let Some(router) = router {
        config.router = router;
    }

    if let Some(enabled) = enabled {
        config.enabled = enabled;
    }

    config.only_band = only_band.unwrap_or(config.only_band);

    CONFIG.save(deps.storage, &config)?;

        Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
            QueryMsg::GetPrice { key } => to_binary(&try_query_price(deps, key)?),
            QueryMsg::GetPrices { keys } => {
                let mut prices = vec![];
                for key in keys {
                    prices.push(try_query_price(deps, key)?);
                }
                to_binary(&prices)
            }
        },
        BLOCK_SIZE,
    )
}

fn try_query_price(
    deps: Deps,
    key: String,
) -> StdResult<OraclePrice> {
    let config = CONFIG.load(deps.storage)?;

    let primary_token: Contract = PRIMARY_TOKEN.load(deps.storage)?;

    let primary_info = PRIMARY_INFO.load(deps.storage)?;

    // Simulate trade 1 primary -> 1 base
    let sim: EstimatedPriceResponse = ShadeSwapQueryMsg::GetEstimatedPrice {
        offer: TokenAmount {
            amount: Uint128::from(10u128.pow(primary_info.decimals.into())),
            token: TokenType::CustomToken {
                contract_addr: primary_token.address,
                token_code_hash: primary_token.code_hash,
            },
        },
    }
    .query(
        &deps.querier,
        config.pair.code_hash.clone(),
        config.pair.address.clone(),
    )?;

    // Normalize to 'rate * 10^18'
    let base_info = BASE_INFO.load(deps.storage)?;
    let exchange_rate = normalize_price(sim.estimated_price, base_info.decimals);

    // Query router for base_peg/USD
    let base_usd_price = if config.only_band {
        query_band_price(&config.router, &deps.querier, config.base_peg)?
    } else {
        query_price(&config.router, &deps.querier, config.base_peg)?
    };

    // Translate price to primary/USD
    let price = base_usd_price
        .data
        .rate
        .multiply_ratio(exchange_rate, 10u128.pow(18));

    Ok(OraclePrice::new(
        key,
        ReferenceData {
            rate: price,
            last_updated_base: base_usd_price.data.last_updated_base,
            last_updated_quote: base_usd_price.data.last_updated_quote,
        },
    ))
}
