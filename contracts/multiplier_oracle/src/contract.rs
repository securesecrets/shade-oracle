use cosmwasm_std::{entry_point, Decimal256, QuerierWrapper, StdError, Uint256};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::msg::{Config, ExecuteMsg, InstantiateMsg, QueryMsg};
use shade_oracles::better_secret_math::common::exp10;
use shade_oracles::core::{pad_query_result, validate_admin, AdminPermissions};
use shade_oracles::interfaces::common::{OraclePrice, OracleQuery, PriceResponse};
use shade_oracles::interfaces::providers::ReferenceData;
use shade_oracles::ssp::Item;
use shade_toolkit::{Contract, Query, BLOCK_SIZE};

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

    let config = Config {
        admin_auth,
        router,
        enabled: true,
        base_key: msg.base_key,
        multiplier: msg.multiplier,
        price_key: msg.price_key,
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
            admin_auth,
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
        )))
    }
}

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
        data: ReferenceData {
            rate: ((Decimal256::from_ratio(underlying_price.data.rate, exp10(18).as_u128())
                * multiplier)
                * Uint256::from(exp10(18).as_u128())),

            last_updated_base: underlying_price.data.last_updated_base,
            last_updated_quote: underlying_price.data.last_updated_quote,
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
