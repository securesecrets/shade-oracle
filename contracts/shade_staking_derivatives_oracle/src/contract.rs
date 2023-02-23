use cosmwasm_std::{
    entry_point, to_binary, Deps, Env, QuerierWrapper, Response, StdResult, Storage,
};
use cosmwasm_std::{DepsMut, MessageInfo, QueryResponse, Uint128};
use shade_oracles::better_secret_math::common::exp10;
use shade_oracles::core::{pad_handle_result, pad_query_result};
use shade_oracles::interfaces::common::config::{CommonConfig, CommonConfigResponse};
use shade_oracles::interfaces::common::{OraclePrice, PriceResponse, PricesResponse};
use shade_oracles::ssp::ItemStorage;
use shade_oracles::{create_attr_action, BLOCK_SIZE};
use shade_oracles::{
    interfaces::derivatives::generic::*, interfaces::providers::ReferenceData,
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    let oracle = StakingDerivativesOracle::load(deps.storage)?;

    pad_query_result(
        match msg {
            QueryMsg::GetPrice { key } => {
                oracle.config.require_enabled()?;
                to_binary(&query_price(
                    &oracle,
                    &env,
                    deps.storage,
                    &deps.querier,
                    key,
                )?)
            }
            QueryMsg::GetPrices { keys } => {
                oracle.config.require_enabled()?;
                to_binary(&query_prices(
                    &oracle,
                    &env,
                    deps.storage,
                    &deps.querier,
                    keys,
                )?)
            }
            QueryMsg::GetConfig {} => to_binary(&query_config(deps.storage, oracle)?),
            QueryMsg::GetDerivatives {} => to_binary(&query_derivatives(deps.storage)?),
        },
        BLOCK_SIZE,
    )
}

pub fn query_price(
    oracle: &StakingDerivativesOracle,
    env: &Env,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
    key: String,
) -> StdResult<PriceResponse> {
    let original_key = key.clone();
    let (is_rate, key) = StakingDerivativesOracle::process_key(&key);
    let data = StakingDerivativesOracle::get_derivative_data_resp(&key, storage)?;
    let resp =
        ShadeStakingDerivative::query_staking_info(querier, &data.staking_derivative.contract)?;

    let staking_derivative_price_precision =
        Uint128::new(10u128.pow(data.staking_derivative.decimals.into()));
    let rate = resp.price;
    let normalized_rate =
        rate.multiply_ratio(exp10(18).as_u128(), staking_derivative_price_precision);

    let now = env.block.time.seconds();
    let data = if is_rate {
        ReferenceData::new(normalized_rate.into(), now, now)
    } else {
        let underlying_price = data
            .staking_derivative
            .get_price(querier, &oracle.config.router)?;

        let price = underlying_price
            .data()
            .rate
            .multiply_ratio(normalized_rate, exp10(18).as_u128());
        ReferenceData::new(
            price,
            underlying_price.data().last_updated_base,
            underlying_price.data().last_updated_quote,
        )
    };

    Ok(OraclePrice::new(original_key, data))
}

pub fn query_prices(
    oracle: &StakingDerivativesOracle,
    env: &Env,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
    keys: Vec<String>,
) -> StdResult<PricesResponse> {
    let mut prices = vec![];
    for key in keys {
        prices.push(query_price(oracle, env, storage, querier, key)?);
    }
    Ok(prices)
}

pub fn query_config(
    storage: &dyn Storage,
    oracle: StakingDerivativesOracle,
) -> StdResult<CommonConfigResponse> {
    let mut resp = oracle.config.get_resp(storage)?;
    let mut supported_keys = vec![];
    for key in &resp.supported_keys {
        supported_keys.push(key.to_string());
        supported_keys.push(StakingDerivativesOracle::create_rate_key(key));
    }
    resp.supported_keys = supported_keys;
    Ok(resp)
}

pub fn query_derivatives(storage: &dyn Storage) -> StdResult<DerivativesResponse> {
    StakingDerivativesOracle::get_supported_derivatives(storage)
}
