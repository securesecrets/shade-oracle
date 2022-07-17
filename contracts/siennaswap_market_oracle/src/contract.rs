use cosmwasm_std::{
    to_binary, MessageInfo, Env, Deps, Response, DepsMut,
    QueryRequest, StdError, StdResult, QueryResponse, entry_point, Uint128,
};
use shade_oracles::{
    pad_handle_result, pad_query_result, ResponseStatus, Contract, BLOCK_SIZE,
    interfaces::band::ReferenceData,
    common::{
        is_disabled,
        querier::{query_token_info, verify_admin, query_band_price, query_price},
        HandleAnswer, OraclePrice, OracleQuery, normalize_price
    },
    interfaces::lp::{
        siennaswap::market::{Config, ExecuteMsg, InstantiateMsg},
    },
    protocols::siennaswap::{
        SiennaDexTokenType, SiennaSwapExchangeQueryMsg, SiennaSwapPairInfoResponse,
        SimulationResponse, TokenTypeAmount
    },
    storage::Item, Query, snip20::helpers::TokenInfo,
};

const CONFIG: Item<Config> = Item::new("config");

const PRIMARY_TOKEN: Item<Contract> = Item::new("primary_token");
const PRIMARY_INFO: Item<TokenInfo> = Item::new("primary_info");
const BASE_INFO: Item<TokenInfo> = Item::new("base_info");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let pair_info_response: SiennaSwapPairInfoResponse = SiennaSwapExchangeQueryMsg::PairInfo
        .query(
            &deps.querier,
            &msg.pair,
        )?;

    let tokens: [Contract; 2] = vec![
        pair_info_response.pair_info.pair.token_0,
        pair_info_response.pair_info.pair.token_1,
    ]
    .iter()
    .filter_map(|t| match t {
        SiennaDexTokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => Some(Contract {
            address: contract_addr.clone(),
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
        .map(|t| query_token_info(t, &deps.querier).ok().unwrap())
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
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    verify_admin(&config.router, deps.as_ref(), info.sender.clone())?;

    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateConfig {
                router,
                enabled,
                only_band,
            } => try_update_config(deps, &info, router, enabled, only_band),
        },
        BLOCK_SIZE,
    )
}

fn try_update_config(
    deps: DepsMut,
    info: &MessageInfo,
    router: Option<Contract>,
    enabled: Option<bool>,
    only_band: Option<bool>,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    verify_admin(&config.router, deps.as_ref(), info.sender.clone())?;
    config.router = router.unwrap_or(config.router);
    config.enabled = enabled.unwrap_or(config.enabled);
    config.only_band = only_band.unwrap_or(config.only_band);

    CONFIG.save(deps.storage, &config)?;

        Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: OracleQuery) -> StdResult<QueryResponse> {
    let config = CONFIG.load(deps.storage)?;
    pad_query_result(
        match msg {
            OracleQuery::GetConfig {} => to_binary(&config),
            OracleQuery::GetPrice { key } => to_binary(&try_query_price(deps, &config, key)?),
            OracleQuery::GetPrices { keys } => {
                let mut prices = vec![];
                for key in keys {
                    prices.push(try_query_price(deps, &config, key)?);
                }
                to_binary(&prices)
            }
        },
        BLOCK_SIZE,
    )
}

fn try_query_price(
    deps: Deps,
    config: &Config,
    key: String,
) -> StdResult<OraclePrice> {
    is_disabled(config.enabled)?;

    let primary_token: Contract = PRIMARY_TOKEN.load(deps.storage)?;
    let primary_info = PRIMARY_INFO.load(deps.storage)?;

    // Simulate trade 1 primary -> 1 base
    let sim: SimulationResponse = SiennaSwapExchangeQueryMsg::SwapSimulation {
        offer: TokenTypeAmount {
            amount: Uint128::from(10u128.pow(primary_info.decimals.into())),
            token: SiennaDexTokenType::CustomToken {
                contract_addr: primary_token.address,
                token_code_hash: primary_token.code_hash,
            },
        },
    }
    .query(
        &deps.querier,
        &config.pair,
    )?;

    // Normalize to 'rate * 10^18'
    let base_info = BASE_INFO.load(deps.storage)?;
    let exchange_rate = normalize_price(sim.return_amount, base_info.decimals);

    // Query router for base_peg/USD
    let base_usd_price = if config.only_band {
        query_band_price(&config.router, &deps.querier, config.base_peg.clone())?
    } else {
        query_price(&config.router, &deps.querier, config.base_peg.clone())?
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
