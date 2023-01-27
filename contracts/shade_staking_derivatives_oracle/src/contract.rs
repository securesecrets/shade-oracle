use cosmwasm_std::{
    entry_point, to_binary, Deps, Env, QuerierWrapper, Response, StdResult, Storage,
};
use cosmwasm_std::{DepsMut, MessageInfo, QueryResponse, Uint128};
use shade_oracles::core::{pad_handle_result, pad_query_result};
use shade_oracles::interfaces::common::config::{CommonConfig, CommonConfigResponse};
use shade_oracles::interfaces::common::{OraclePrice, PriceResponse, PricesResponse};
use shade_oracles::ssp::ItemStorage;
use shade_oracles::{create_attr_action, BLOCK_SIZE};
use shade_oracles::{
    interfaces::band::ReferenceData, interfaces::derivatives::generic::*,
    protocols::shade_staking_derivatives::ShadeStakingDerivative,
};

create_attr_action!("shade-staking-derivatives-oracle_");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = CommonConfig::init(deps.api, deps.storage, msg.router)?;
    StakingDerivativesOracle { config }.save(deps.storage)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let mut oracle = StakingDerivativesOracle::load(deps.storage)?;
    let resp = Response::new();
    let resp = match msg {
        ExecuteMsg::SetStatus(status) => {
            oracle.config.require_admin(&deps.querier, info)?;
            oracle.config.update_config(deps.api, Some(status), None)?;
            oracle.save(deps.storage)?;
            resp.add_attributes(vec![attr_action!("set_status")])
        }
        _ => {
            oracle.config.require_enabled()?;
            match msg {
                ExecuteMsg::SetDerivatives(data) => {
                    oracle.config.require_admin(&deps.querier, info)?;
                    for item in data {
                        oracle.validate_and_set_derivative_data(
                            deps.storage,
                            deps.api,
                            &deps.querier,
                            item,
                        )?;
                    }
                    resp.add_attributes(vec![attr_action!("set_derivatives")])
                }
                ExecuteMsg::RemoveDerivatives(keys) => {
                    oracle.config.require_admin(&deps.querier, info)?;
                    StakingDerivativesOracle::remove_keys(deps.storage, keys)?;
                    resp.add_attributes(vec![attr_action!("remove_derivatives")])
                }
                ExecuteMsg::UpdateAssets(assets) => {
                    oracle.config.require_admin(&deps.querier, info)?;
                    for asset in assets {
                        oracle.update_asset_symbol(deps.storage, deps.api, &deps.querier, asset)?;
                    }
                    resp.add_attributes(vec![attr_action!("update_assets")])
                }
                ExecuteMsg::UpdateConfig(new_router) => {
                    oracle.config.require_admin(&deps.querier, info)?;
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
    let oracle = StakingDerivativesOracle::load(deps.storage)?;

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
            QueryMsg::GetDerivatives {} => to_binary(&query_derivatives(deps.storage)?),
        },
        BLOCK_SIZE,
    )
}

pub fn query_price(
    oracle: &StakingDerivativesOracle,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
    key: String,
) -> StdResult<PriceResponse> {
    let data = StakingDerivativesOracle::get_derivative_data_resp(&key, storage)?;
    // Simulate trade 1 primary -> 1 base
    let sim =
        ShadeStakingDerivative::query_staking_info(querier, &data.staking_derivative.contract)?;

    let staking_derivative_price_precision =
        Uint128::new(10u128.pow(data.staking_derivative.decimals.into()));

    let underlying_price = data
        .staking_derivative
        .get_price(querier, &oracle.config.router)?;

    let price = underlying_price
        .data()
        .rate
        .multiply_ratio(sim.price, staking_derivative_price_precision);

    Ok(OraclePrice::new(
        key,
        ReferenceData {
            rate: price,
            last_updated_base: underlying_price.data().last_updated_base,
            last_updated_quote: underlying_price.data().last_updated_quote,
        },
    ))
}

pub fn query_prices(
    oracle: &StakingDerivativesOracle,
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
    oracle: StakingDerivativesOracle,
) -> StdResult<CommonConfigResponse> {
    let supported_keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
    Ok(CommonConfigResponse {
        config: oracle.config,
        supported_keys,
    })
}

pub fn query_derivatives(storage: &dyn Storage) -> StdResult<DerivativesResponse> {
    StakingDerivativesOracle::get_supported_derivatives(storage)
}
