use std::cmp::min;

use cosmwasm_std::{
    entry_point, to_binary, Deps, Env, QuerierWrapper, Response, StdResult, Storage, Uint256,
};
use cosmwasm_std::{DepsMut, MessageInfo, QueryResponse};
use shade_oracles::core::{pad_handle_result, pad_query_result};
use shade_oracles::ssp::ItemStorage;
use shade_oracles::{
    common::querier::query_price as query_router_price,
    interfaces::common::{
        config::{CommonConfig, CommonConfigResponse},
        OraclePrice, PriceResponse, PricesResponse,
    },
};
use shade_oracles::{create_attr_action, BLOCK_SIZE};
use shade_oracles::{interfaces::band::ReferenceData, interfaces::derivatives::stride::*};

create_attr_action!("stride-staking-derivatives-oracle_");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = CommonConfig::init(deps.api, deps.storage, msg.router)?;
    StrideStakingDerivativesOracle { config }.save(deps.storage)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let mut oracle = StrideStakingDerivativesOracle::load(deps.storage)?;
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
            let now = env.block.time.seconds();
            match msg {
                ExecuteMsg::SetDerivatives(data) => {
                    oracle.config.require_admin(&deps.querier, info)?;
                    oracle.set_derivatives(deps.storage, &deps.querier, now, data)?;
                    resp.add_attributes(vec![attr_action!("set_derivatives")])
                }
                ExecuteMsg::RemoveDerivatives(keys) => {
                    oracle.config.require_admin(&deps.querier, info)?;
                    StrideStakingDerivativesOracle::remove_keys(deps.storage, keys)?;
                    resp.add_attributes(vec![attr_action!("remove_derivatives")])
                }
                ExecuteMsg::UpdateDerivatives(update) => match update {
                    DerivativeUpdates::Rates(rates) => {
                        oracle.config.require_permission(
                            &deps.querier,
                            info,
                            BotPermission::UpdateRates,
                        )?;
                        StrideStakingDerivativesOracle::update_rates(deps.storage, now, rates)?;
                        resp.add_attribute_plaintext(
                            "action",
                            "shade_staking_derivatives_oracle_exchange_rate_update",
                        )
                    }
                    DerivativeUpdates::APY(apys) => {
                        oracle.config.require_permission(
                            &deps.querier,
                            info,
                            BotPermission::UpdateAPY,
                        )?;
                        StrideStakingDerivativesOracle::update_apys(deps.storage, now, apys)?;
                        resp.add_attribute_plaintext(
                            "action",
                            "shade_staking_derivatives_oracle_apy_update",
                        )
                    }
                    DerivativeUpdates::Config(configs) => {
                        oracle.config.require_admin(&deps.querier, info)?;
                        oracle.update_derivatives(&deps.querier, deps.storage, configs)?;
                        resp.add_attribute_plaintext(
                            "action",
                            "shade_staking_derivatives_oracle_config_update",
                        )
                    }
                },
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
    let oracle = StrideStakingDerivativesOracle::load(deps.storage)?;

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
    oracle: &StrideStakingDerivativesOracle,
    env: &Env,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
    key: String,
) -> StdResult<PriceResponse> {
    let original_key = key.clone();
    let (is_rate, key) = StrideStakingDerivativesOracle::process_key(&key);
    let stored_data = StrideStakingDerivativesOracle::DERIVATIVES.load(storage, &key)?;

    let now = env.block.time.seconds();
    stored_data.rate.require_fresh(now)?;

    let rate = stored_data.rate;

    let data = if is_rate {
        ReferenceData::new(
            rate.value.atomics().try_into()?,
            rate.last_updated,
            rate.last_updated,
        )
    } else {
        let underlying_price =
            query_router_price(&oracle.config.router, querier, &stored_data.underlying_key)?;
        let price = rate.value * Uint256::from_uint128(underlying_price.data().rate);
        ReferenceData::new(
            price.try_into()?,
            min(underlying_price.data().last_updated_base, rate.last_updated),
            underlying_price.data().last_updated_quote,
        )
    };

    Ok(OraclePrice::new(original_key, data))
}

pub fn query_prices(
    oracle: &StrideStakingDerivativesOracle,
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
    oracle: StrideStakingDerivativesOracle,
) -> StdResult<CommonConfigResponse> {
    let mut resp = oracle.config.get_resp(storage)?;
    let mut supported_keys = vec![];
    for key in &resp.supported_keys.clone() {
        supported_keys.push(key.to_string());
        supported_keys.push(StrideStakingDerivativesOracle::create_rate_key(key));
    }
    resp.supported_keys = supported_keys;
    Ok(resp)
}

pub fn query_derivatives(storage: &dyn Storage) -> StdResult<DerivativesResponse> {
    StrideStakingDerivativesOracle::get_supported_derivatives(storage)
}
