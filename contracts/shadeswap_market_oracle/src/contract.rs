use cosmwasm_std::{entry_point, Deps, Env, Response, StdError, StdResult};
use cosmwasm_std::{DepsMut, MessageInfo, QueryResponse, Uint128};
use shade_oracles::core::Query;
use shade_oracles::create_attr_action;
use shade_oracles::interfaces::common::config::CommonConfig;
use shade_oracles::ssp::ItemStorage;
use shade_oracles::{
    common::querier::{query_band_price, query_price, query_token_info},
    core::Contract,
    interfaces::band::ReferenceData,
    interfaces::dex::market::*,
    protocols::shadeswap::ShadeSwapQuerier,
};

impl Oracle for ShadeswapMarketOracle {
    fn try_query_price(
        &self,
        deps: Deps,
        _env: &Env,
        key: String,
        config: &shade_oracles::common::CommonConfig,
    ) -> StdResult<OraclePrice> {
        let primary_token = PRIMARY_TOKEN.load(deps.storage)?;
        let market_config = MarketData::load(deps.storage)?;
        let primary_info = PRIMARY_INFO.load(deps.storage)?;

        // Simulate trade 1 primary -> 1 base
        let sim: EstimatedPriceResponse = ShadeSwapQueryMsg::SwapSimulation {
            offer: TokenAmount {
                amount: Uint128::from(10u128.pow(primary_info.decimals.into())),
                token: TokenType::CustomToken {
                    contract_addr: primary_token.address,
                    token_code_hash: primary_token.code_hash,
                    oracle_key: _,
                },
            },
            exclude_fee: Some(true),
        }
        .query(&deps.querier, &market_config.pair)?;

        // Normalize to 'rate * 10^18'
        let base_info = BASE_INFO.load(deps.storage)?;
        let exchange_rate = normalize_price_uint128(sim.estimated_price, base_info.decimals)?;

        // Query router for base_peg/USD
        let base_usd_price = if config.only_band {
            query_band_price(&config.router, &deps.querier, market_config.base_peg)?
        } else {
            query_price(&config.router, &deps.querier, market_config.base_peg)?
        };

        // Translate price to primary/USD
        let price = base_usd_price
            .data()
            .rate
            .multiply_ratio(exchange_rate, 10u128.pow(18));

        Ok(OraclePrice::new(
            key,
            ReferenceData {
                rate: price,
                last_updated_base: base_usd_price.data().last_updated_base,
                last_updated_quote: base_usd_price.data().last_updated_quote,
            },
        ))
    }
}

create_attr_action!("shadeswap-market-oracle_");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = CommonConfig::init(deps.api, msg.router)?;
    LiquidityPairMarketOracle { config }.save(deps.storage)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let mut oracle = LiquidityPairMarketOracle::load(deps.storage)?;
    let resp = Response::new();
    oracle.config.require_admin(&deps.querier, info)?;
    let resp = match msg {
        ExecuteMsg::SetStatus(status) => {
            oracle.config.update_config(deps.api, Some(status), None)?;
            oracle.save(deps.storage)?;
            resp.add_attributes(vec![attr_action!("set_status")])
        }
        _ => {
            oracle.config.require_enabled()?;
            match msg {
                ExecuteMsg::SetKey {
                    key,
                    base_token,
                    target_token,
                    pair,
                } => {}
                ExecuteMsg::SetPairs(data) => {
                    for item in data {
                        let valid_data = oracle.validate_and_set_pair_data(
                            deps.storage,
                            deps.api,
                            &deps.querier,
                            item,
                        )?;
                        let pair_info_response =
                            ShadeSwapQuerier::query_pair_info(&deps.querier, &valid_data.pair)?;
                        let actual_pair = pair_info_response.pair;
                        actual_pair.0.require_address_eq(&valid_data.token_0)?;
                        actual_pair.1.require_address_eq(&valid_data.token_1)?;
                    }
                    resp.add_attributes(vec![attr_action!("set_pairs")])
                }
                ExecuteMsg::RemovePairs(keys) => {
                    LiquidityPairOracle::remove_keys(deps.storage, keys)?;
                    resp.add_attributes(vec![attr_action!("remove_pairs")])
                }
                ExecuteMsg::UpdateAssets(assets) => {
                    for asset in assets {
                        oracle.update_asset_symbol(deps.storage, deps.api, &deps.querier, asset)?;
                    }
                    resp.add_attributes(vec![attr_action!("update_assets")])
                }
                ExecuteMsg::UpdateConfig(new_router) => {
                    oracle
                        .config
                        .update_config(deps.api, None, Some(new_router))?;
                    oracle.save(deps.storage)?;
                    resp.add_attributes(vec![attr_action!("update_config")])
                }
                _ => panic!("Code should never go here."),
            }
        }
    };
    pad_handle_result(Ok(resp), BLOCK_SIZE)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    let oracle = LiquidityPairOracle::load(deps.storage)?;
    oracle.config.require_enabled()?;

    pad_query_result(
        match msg {
            QueryMsg::GetPrice { key } => {
                to_binary(&query_price(&oracle, deps.storage, &deps.querier, key)?)
            }
            QueryMsg::GetPrices { keys } => {
                to_binary(&query_prices(&oracle, deps.storage, &deps.querier, keys)?)
            }
            QueryMsg::GetConfig {} => to_binary(&query_config(deps.storage, oracle)?),
            QueryMsg::GetPairs {} => to_binary(&query_pairs(deps.storage)),
        },
        BLOCK_SIZE,
    )
}

pub fn query_price(
    oracle: &LiquidityPairOracle,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
    key: String,
) -> StdResult<PriceResponse> {
    let data = LiquidityPairOracle::get_pair_data_resp(&key, storage)?;
    let prices = query_router_prices(
        &oracle.config.router,
        querier,
        &vec![
            data.token_0.quote_symbol.clone(),
            data.token_1.quote_symbol.clone(),
        ],
    )?;
    let pair_resp = ShadeSwapQuerier::query_pair_info(querier, &data.pair)?;

    if pair_resp.is_stableswap() {
        Err(StdError::generic_err(format!(
            "Pair {} is a stableswap pair which is not supported by this oracle.",
            data.pair.address
        )))
    } else {
        let lp_token_info = query_token_info(&pair_resp.liquidity_token, &querier)?;

        let reserves_0 = pair_resp.amount_0;
        let reserves_1 = pair_resp.amount_1;

        let data = LiquidityPairOracle::calculate_lp_token_spot_rate(
            data,
            lp_token_info,
            reserves_0,
            reserves_1,
            &[&prices[0], &prices[1]],
        )?;
        Ok(OraclePrice::new(key.to_string(), data))
    }
}

pub fn query_prices(
    oracle: &LiquidityPairOracle,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
    keys: Vec<String>,
) -> StdResult<PricesResponse> {
    let mut prices = vec![];
    for key in keys {
        prices.push(query_price(oracle, storage, querier, key)?);
    }
    Ok(prices)
}

pub fn query_config(
    storage: &dyn Storage,
    oracle: LiquidityPairOracle,
) -> StdResult<CommonConfigResponse> {
    let supported_keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
    Ok(CommonConfigResponse {
        config: oracle.config,
        supported_keys,
    })
}

pub fn query_pairs(storage: &dyn Storage) -> StdResult<PairsResponse> {
    Ok(LiquidityPairOracle::get_supported_pairs(storage)?)
}
