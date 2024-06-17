use cosmwasm_std::{entry_point, QuerierWrapper, StdError, Uint128, Uint256};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use shade_oracles::better_secret_math::common::exp10;
use shade_oracles::core::{pad_query_result, validate_admin, AdminPermissions};
use shade_oracles::interfaces::common::{OraclePrice, OracleQuery, PriceResponse};
use shade_oracles::interfaces::providers::ReferenceData;
use shade_oracles::ssp::Item;
use shade_toolkit::{Contract, Query, BLOCK_SIZE};

use crate::{dshd, msg::*};

// Key used to query router
pub const UNDERLYING_KEY: &str = "SHD";
// Key for the "price" (underlying * redemption_rate)
pub const PRICE_KEY: &str = "Shade Derivative";
// Key for the redemption rate
pub const RATE_KEY: &str = "Shade Derivative Rate";

// Storage
const CONFIG: Item<Config> = Item::new("config");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let admin_auth = msg.admin_auth.validate(deps.api)?;
    let router = msg.router.validate(deps.api)?;
    let dshd = msg.dshd.validate(deps.api)?;

    let config = Config {
        admin_auth,
        router,
        dshd,
        enabled: true,
    };

    CONFIG.save(deps.storage, &config)?;

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
            dshd,
            admin_auth,
            enabled,
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
            if let Some(dshd) = dshd {
                config.dshd = dshd.validate(deps.api)?;
                resp = resp.add_attribute("dshd", config.dshd.address.clone());
            }
            if let Some(enabled) = enabled {
                config.enabled = enabled;
                resp = resp.add_attribute("enabled", config.enabled.to_string());
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
            QueryMsg::GetPrice { key } => to_binary(&query_oracle_price_by_key(
                &deps,
                &env,
                key,
                &config.router,
                &config.dshd,
            )?),
            QueryMsg::GetPrices { keys } => {
                let mut results = vec![];
                for key in keys {
                    results.push(query_oracle_price_by_key(
                        &deps,
                        &env,
                        key,
                        &config.router,
                        &config.dshd,
                    )?);
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
    router: &Contract,
    dshd: &Contract,
) -> StdResult<OraclePrice> {
    if key == PRICE_KEY.to_string() {
        query_price(deps, env, router, dshd)
    } else if key == RATE_KEY.to_string() {
        query_rate(deps, env, dshd)
    } else {
        Err(StdError::generic_err(format!(
            "Invalid Key, expected one of {}, {}",
            PRICE_KEY, RATE_KEY
        )))
    }
}

fn query_price(
    deps: &Deps,
    env: &Env,
    router: &Contract,
    dshd: &Contract,
) -> StdResult<OraclePrice> {
    let underlying_price = query_router_price(&router, &deps.querier, UNDERLYING_KEY.to_string())?;
    let rate = query_rate(deps, env, dshd)?;
    Ok(OraclePrice {
        key: PRICE_KEY.to_string(),
        data: ReferenceData {
            rate: (underlying_price.data.rate * rate.data.rate)
                / Uint256::from(exp10(18).as_u128()),
            last_updated_base: underlying_price.data.last_updated_base,
            last_updated_quote: underlying_price.data.last_updated_quote,
        },
    })
}

fn query_rate(deps: &Deps, env: &Env, dshd: &Contract) -> StdResult<OraclePrice> {
    let staking_info = query_staking_info(&dshd, &deps.querier)?;
    let now = env.block.time.seconds();
    // normalize from 10^6 to 10^18
    Ok(OraclePrice {
        key: RATE_KEY.to_string(),
        data: ReferenceData {
            // price is in utkn (8 decimal SHD), upscaling by 10^10 to get 10^18
            rate: Uint256::from(staking_info.price * Uint128::new(exp10(10).as_u128())),
            last_updated_base: now,
            last_updated_quote: now,
        },
    })
}

pub fn query_router_price(
    router: &Contract,
    querier: &QuerierWrapper,
    key: impl Into<String>,
) -> StdResult<PriceResponse> {
    OracleQuery::GetPrice { key: key.into() }.query(querier, router)
}

pub fn query_staking_info(
    dshd: &Contract,
    querier: &QuerierWrapper,
) -> StdResult<dshd::StakingInfoResponse> {
    match (dshd::QueryMsg::StakingInfo {}.query(querier, dshd)?) {
        dshd::QueryResponse::StakingInfo {
            unbonding_time,
            bonded_shd,
            rewards,
            total_derivative_token_supply,
            price,
            fee_info,
            status,
        } => Ok(dshd::StakingInfoResponse {
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
