use cosmwasm_std::{entry_point, QuerierWrapper, Storage};
use cosmwasm_std::{
    to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdResult,
};
use shade_oracles::core::pad_handle_result;
use shade_oracles::create_attr_action;
use shade_oracles::interfaces::common::config::{CommonConfig, CommonConfigResponse};
use shade_oracles::interfaces::common::{OraclePrice, PriceResponse, PricesResponse};
use shade_oracles::protocols::siennaswap::SiennaSwapQuerier;
use shade_oracles::{
    common::querier::{query_prices as query_router_prices, query_token_info},
    core::pad_query_result,
    interfaces::dex::pair::*,
    ssp::ItemStorage,
    BLOCK_SIZE,
};

create_attr_action!("siennaswap-spot-oracle_");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = CommonConfig::init(deps.api, msg.router)?;
    LiquidityPairOracle { config }.save(deps.storage)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let mut oracle = LiquidityPairOracle::load(deps.storage)?;
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
                ExecuteMsg::SetPairs(data) => {
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
                        actual_pair
                            .token_0
                            .require_address_eq(&valid_data.token_0)?;
                        actual_pair
                            .token_1
                            .require_address_eq(&valid_data.token_1)?;
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
    oracle: &LiquidityPairOracle,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
    key: String,
) -> StdResult<PriceResponse> {
    let data = LiquidityPairOracle::get_pair_data_resp(&key, storage)?;
    let prices = query_router_prices(
        &oracle.config.router,
        querier,
        &[
            data.token_0.quote_symbol.clone(),
            data.token_1.quote_symbol.clone(),
        ],
    )?;
    let pair_resp = SiennaSwapQuerier::query_pair_info(querier, &data.pair)?.pair_info;
    let lp_token_info = query_token_info(&pair_resp.liquidity_token, querier)?;

    let reserves_0 = pair_resp.amount_0;
    let reserves_1 = pair_resp.amount_1;

    let data = LiquidityPairOracle::calculate_lp_token_spot_rate(
        data,
        lp_token_info,
        reserves_0,
        reserves_1,
        &[&prices[0], &prices[1]],
    )?;

    Ok(OraclePrice::new(key, data))
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
    LiquidityPairOracle::get_supported_pairs(storage)
}
