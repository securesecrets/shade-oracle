use cosmwasm_std::{entry_point, Decimal256, QuerierWrapper, StdError, Uint256};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::msg::{Config, ExecuteMsg, InstantiateMsg, QueryMsg};
use shade_oracles::better_secret_math::common::exp10;
use shade_oracles::core::{pad_query_result, validate_admin, AdminPermissions};
use shade_oracles::interfaces::common::{OraclePrice, OracleQuery, PriceResponse};
use shade_oracles::interfaces::providers::ReferenceData;
use shade_oracles::ssp::Item;
use shade_toolkit::{Contract, Query, BLOCK_SIZE};

<<<<<<<< HEAD:contracts/derivative_oracle/src/contract.rs
use crate::{derivative, msg::*};

/*
========
>>>>>>>> origin/develop:contracts/multiplier_oracle/src/contract.rs
// Key used to query router
pub const UNDERLYING_KEY: &str = "SHD";
// Key for the "price" (underlying * redemption_rate)
pub const PRICE_KEY: &str = "Shade Derivative";
// Key for the redemption rate
pub const RATE_KEY: &str = "Shade Derivative Rate";
*/

// Storage
const CONFIG: Item<Config> = Item::new("config");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
<<<<<<<< HEAD:contracts/derivative_oracle/src/contract.rs
    CONFIG.save(
        deps.storage,
        &Config {
            admin_auth: msg.admin_auth.validate(deps.api)?,
            router: msg.router.validate(deps.api)?,
            derivative: msg.derivative.validate(deps.api)?,
            underlying_key: msg.underlying_key,
            underlying_decimals: msg.underlying_decimals,
            price_key: msg.price_key,
            rate_key: msg.rate_key,
            enabled: true,
        },
    )?;
========
    let admin_auth = msg.admin_auth.validate(deps.api)?;
    let router = msg.router.validate(deps.api)?;

    let config = Config {
        admin_auth,
        router,
        enabled: true,
        base_key: msg.base_key,
        multiplier: msg.multiplier,
        price_key: msg.price_key,
    };

    CONFIG.save(deps.storage, &config)?;
>>>>>>>> origin/develop:contracts/multiplier_oracle/src/contract.rs

    Ok(Response::default())
}

fn require_enabled(config: &Config) -> StdResult<()> {
    if !config.enabled {
        return Err(StdError::generic_err("Contract is disabled"));
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    match msg {
        ExecuteMsg::UpdateConfig {
            router,
<<<<<<<< HEAD:contracts/derivative_oracle/src/contract.rs
            derivative,
========
>>>>>>>> origin/develop:contracts/multiplier_oracle/src/contract.rs
            admin_auth,
            underlying_key,
            underlying_decimals,
            price_key,
            rate_key,
            enabled,
            multiplier,
        } => {
            validate_admin(
                &deps.querier,
                AdminPermissions::OraclesAdmin,
                info.sender,
                &config.admin_auth,
            )?;
            let mut resp = Response::default();
            if let Some(router) = router {
                config.router = router.validate(deps.api)?;
                resp = resp.add_attribute("router", config.router.address.clone());
            }
            if let Some(admin_auth) = admin_auth {
                config.admin_auth = admin_auth.validate(deps.api)?;
                resp = resp.add_attribute("admin_auth", config.admin_auth.address.clone());
            }
<<<<<<<< HEAD:contracts/derivative_oracle/src/contract.rs
            if let Some(derivative) = derivative {
                config.derivative = derivative.validate(deps.api)?;
                resp = resp.add_attribute("derivative", config.derivative.address.clone());
            }
            if let Some(underlying_key) = underlying_key {
                config.underlying_key = underlying_key;
                resp = resp.add_attribute("underlying_key", config.underlying_key.clone());
            }
            if let Some(underlying_decimals) = underlying_decimals {
                config.underlying_decimals = underlying_decimals;
                resp = resp.add_attribute(
                    "underlying_decimals",
                    config.underlying_decimals.to_string(),
                );
            }
            if let Some(price_key) = price_key {
                config.price_key = price_key;
                resp = resp.add_attribute("price_key", config.price_key.clone());
            }
            if let Some(rate_key) = rate_key {
                config.rate_key = rate_key;
                resp = resp.add_attribute("rate_key", config.rate_key.clone());
            }
========
>>>>>>>> origin/develop:contracts/multiplier_oracle/src/contract.rs
            if let Some(enabled) = enabled {
                config.enabled = enabled;
                resp = resp.add_attribute("enabled", config.enabled.to_string());
            }
            if let Some(multiplier) = multiplier {
                config.multiplier = multiplier;
                resp = resp.add_attribute("multiplier", config.multiplier.to_string());
            }
            CONFIG.save(deps.storage, &config)?;
            Ok(resp.add_attribute("action", "update_config"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    require_enabled(&config)?;
    pad_query_result(
        match msg {
<<<<<<<< HEAD:contracts/derivative_oracle/src/contract.rs
            QueryMsg::GetPrice { key } => {
                to_binary(&query_oracle_price_by_key(&deps, &env, key, &config)?)
            }
            QueryMsg::GetPrices { keys } => {
                let mut results = vec![];
                for key in keys {
                    results.push(query_oracle_price_by_key(&deps, &env, key, &config)?);
========
            QueryMsg::GetPrice { key } => to_binary(&query_oracle_price_by_key(
                &deps,
                &env,
                key,
                &config.router,
                &config.base_key,
                &config.price_key,
                &config.multiplier,
            )?),
            QueryMsg::GetPrices { keys } => {
                let mut results = vec![];
                for key in keys {
                    results.push(query_oracle_price_by_key(
                        &deps,
                        &env,
                        key,
                        &config.router,
                        &config.base_key,
                        &config.price_key,
                        &config.multiplier,
                    )?);
>>>>>>>> origin/develop:contracts/multiplier_oracle/src/contract.rs
                }
                to_binary(&results)
            }
            QueryMsg::GetConfig {} => to_binary(&config),
        },
        BLOCK_SIZE,
    )
}

fn query_oracle_price_by_key(
    deps: &Deps,
    env: &Env,
    key: String,
<<<<<<<< HEAD:contracts/derivative_oracle/src/contract.rs
    config: &Config,
) -> StdResult<OraclePrice> {
    if key == config.price_key.to_string() {
        query_price(deps, env, config)
    } else if key == config.rate_key.to_string() {
        query_rate(deps, env, config)
    } else {
        Err(StdError::generic_err(format!(
            "Invalid Key, expected one of {}, {}",
            config.price_key, config.rate_key
========
    router: &Contract,
    base_key: &String,
    price_key: &String,
    multiplier: &Decimal256,
) -> StdResult<OraclePrice> {
    if key == price_key.to_string() {
        query_price(deps, env, router, base_key, price_key, multiplier)
    } else {
        Err(StdError::generic_err(format!(
            "Invalid Key, expected {}",
            price_key,
>>>>>>>> origin/develop:contracts/multiplier_oracle/src/contract.rs
        )))
    }
}

<<<<<<<< HEAD:contracts/derivative_oracle/src/contract.rs
fn query_price(deps: &Deps, env: &Env, config: &Config) -> StdResult<OraclePrice> {
    let underlying_price =
        query_router_price(&config.router, &deps.querier, config.underlying_key.clone())?;
    let rate = query_rate(deps, env, config)?;
    Ok(OraclePrice {
        key: config.price_key.to_string(),
========
fn query_price(
    deps: &Deps,
    env: &Env,
    router: &Contract,
    base_key: &String,
    price_key: &String,
    multiplier: &Decimal256,
) -> StdResult<OraclePrice> {
    let underlying_price = query_router_price(&router, &deps.querier, base_key.to_string())?;
    Ok(OraclePrice {
        key: price_key.to_string(),
>>>>>>>> origin/develop:contracts/multiplier_oracle/src/contract.rs
        data: ReferenceData {
            rate: ((Decimal256::from_ratio(underlying_price.data.rate, exp10(18).as_u128())
                * multiplier)
                * Uint256::from(exp10(18).as_u128())),

            last_updated_base: underlying_price.data.last_updated_base,
            last_updated_quote: underlying_price.data.last_updated_quote,
        },
    })
}

<<<<<<<< HEAD:contracts/derivative_oracle/src/contract.rs
fn query_rate(deps: &Deps, env: &Env, config: &Config) -> StdResult<OraclePrice> {
    let staking_info = query_staking_info(&config.derivative, &deps.querier)?;
    let now = env.block.time.seconds();
    // normalize from 10^6 to 10^18
    Ok(OraclePrice {
        key: config.rate_key.to_string(),
        data: ReferenceData {
            // price is in utkn, upscaling by 10^(18-decimals) to get 10^18
            rate: Uint256::from(
                staking_info.price * Uint128::new(exp10(18 - config.underlying_decimals).as_u128()),
            ),
            last_updated_base: now,
            last_updated_quote: now,
        },
    })
}

========
>>>>>>>> origin/develop:contracts/multiplier_oracle/src/contract.rs
pub fn query_router_price(
    router: &Contract,
    querier: &QuerierWrapper,
    key: impl Into<String>,
) -> StdResult<PriceResponse> {
    OracleQuery::GetPrice { key: key.into() }.query(querier, router)
}
<<<<<<<< HEAD:contracts/derivative_oracle/src/contract.rs

pub fn query_staking_info(
    derivative: &Contract,
    querier: &QuerierWrapper,
) -> StdResult<derivative::StakingInfoResponse> {
    match (derivative::QueryMsg::StakingInfo {}.query(querier, derivative)?) {
        derivative::QueryResponse::StakingInfo {
            unbonding_time,
            bonded_shd,
            rewards,
            total_derivative_token_supply,
            price,
            fee_info,
            status,
        } => Ok(derivative::StakingInfoResponse {
            unbonding_time,
            bonded_shd,
            rewards,
            total_derivative_token_supply,
            price,
            fee_info,
            status,
        }),
    }
}
========
>>>>>>>> origin/develop:contracts/multiplier_oracle/src/contract.rs
