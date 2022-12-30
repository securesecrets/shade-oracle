use cosmwasm_std::{
    entry_point, to_binary, Deps, Env, QuerierWrapper, Response, StdResult, Storage,
};
use cosmwasm_std::{DepsMut, MessageInfo, QueryResponse, Uint128};
use shade_oracles::core::{pad_handle_result, pad_query_result};
use shade_oracles::interfaces::common::config::{CommonConfig, CommonConfigResponse};
use shade_oracles::interfaces::common::{OraclePrice, PriceResponse, PricesResponse};
use shade_oracles::math::TokenMath;
use shade_oracles::ssp::ItemStorage;
use shade_oracles::{create_attr_action, BLOCK_SIZE};
use shade_oracles::{
    interfaces::band::ReferenceData, interfaces::dex::market::*,
    protocols::siennaswap::SiennaSwapQuerier,
};

create_attr_action!("siennaswap-market-oracle_");

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
                ExecuteMsg::SetKeys(data) => {
                    for item in data {
                        let valid_data = oracle.validate_and_set_pair_data(
                            deps.storage,
                            deps.api,
                            &deps.querier,
                            item,
                        )?;
                        let pair_info_response =
                            SiennaSwapQuerier::query_pair_info(&deps.querier, &valid_data.pair)?;
                        let actual_pair = pair_info_response.pair_info.pair;
                        actual_pair.require_has_addresses(
                            &valid_data.base_token,
                            &valid_data.target_token,
                        )?;
                    }
                    resp.add_attributes(vec![attr_action!("set_keys")])
                }
                ExecuteMsg::RemoveKeys(keys) => {
                    LiquidityPairMarketOracle::remove_keys(deps.storage, keys)?;
                    resp.add_attributes(vec![attr_action!("remove_keys")])
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
    let oracle = LiquidityPairMarketOracle::load(deps.storage)?;

    pad_query_result(
        match msg {
            QueryMsg::GetPrice { key } => {
                oracle.config.require_enabled()?;
                to_binary(&query_price(&oracle, deps.storage, &deps.querier, key)?)
            }
            QueryMsg::GetPrices { keys } => {
                oracle.config.require_enabled()?;
                to_binary(&query_prices(&oracle, deps.storage, &deps.querier, keys)?)
            }
            QueryMsg::GetConfig {} => to_binary(&query_config(deps.storage, oracle)?),
            QueryMsg::GetPairs {} => to_binary(&query_pairs(deps.storage)?),
        },
        BLOCK_SIZE,
    )
}

pub fn query_price(
    oracle: &LiquidityPairMarketOracle,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
    key: String,
) -> StdResult<PriceResponse> {
    let pair_data = LiquidityPairMarketOracle::get_pair_data_resp(&key, storage)?;
    // Simulate trade 1 target -> 1 base
    let sim = SiennaSwapQuerier::query_swap_simulation(
        querier,
        &pair_data.pair,
        &pair_data.target_token.contract,
        Uint128::from(10u128.pow(pair_data.target_token.decimals.into())),
    )?;

    // Normalize to 'rate * 10^18'
    let exchange_rate: Uint128 =
        TokenMath::normalize_value(sim.return_amount, pair_data.base_token.decimals)?.into();

    // Query router for base_peg/USD
    let base_usd_price = pair_data
        .base_token
        .get_price(querier, &oracle.config.router)?;

    // Translate price to target/USD
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

pub fn query_prices(
    oracle: &LiquidityPairMarketOracle,
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
    oracle: LiquidityPairMarketOracle,
) -> StdResult<CommonConfigResponse> {
    let supported_keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
    Ok(CommonConfigResponse {
        config: oracle.config,
        supported_keys,
    })
}

pub fn query_pairs(storage: &dyn Storage) -> StdResult<PairsResponse> {
    LiquidityPairMarketOracle::get_supported_pairs(storage)
}
