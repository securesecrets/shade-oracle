use cosmwasm_std::{entry_point, QuerierWrapper, StdError, Uint128, Uint256};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use secret_toolkit::snip20::token_info_query;
use shade_oracles::better_secret_math::common::exp10;
use shade_oracles::core::{pad_query_result, validate_admin, AdminPermissions};
use shade_oracles::interfaces::common::{OraclePrice, OracleQuery, PriceResponse};
use shade_oracles::interfaces::providers::ReferenceData;
use shade_oracles::ssp::{Item, Map};
use shade_toolkit::{Contract, Query, BLOCK_SIZE};

use crate::{money_market, msg::*};

// Storage
const CONFIG: Item<Config> = Item::new("config");
const KEY_VAULT_MAP: Map<String, Addr> = Map::new("key_vault_map");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            admin_auth: msg.admin_auth.validate(deps.api)?,
            router: msg.router.validate(deps.api)?,
            money_market: msg.money_market.validate(deps.api)?,
            enabled: true,
        },
    )?;

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
            money_market,
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
            if let Some(money_market) = money_market {
                config.money_market = money_market.validate(deps.api)?;
                resp = resp.add_attribute("money_market", config.money_market.address.clone());
            }
            if let Some(enabled) = enabled {
                config.enabled = enabled;
                resp = resp.add_attribute("enabled", config.enabled.to_string());
            }
            CONFIG.save(deps.storage, &config)?;
            Ok(resp.add_attribute("action", "update_config"))
        }
        ExecuteMsg::RegisterVault(key_config) => {
            validate_admin(
                &deps.querier,
                AdminPermissions::OraclesAdmin,
                info.sender,
                &config.admin_auth,
            )?;
            KEY_VAULT_MAP.save(
                deps.storage,
                key_config.key,
                &deps.api.addr_validate(&key_config.vault)?,
            )?;
            Ok(Response::default().add_attribute("action", "register_vault"))
        }
        ExecuteMsg::RegisterVaults(key_configs) => {
            validate_admin(
                &deps.querier,
                AdminPermissions::OraclesAdmin,
                info.sender,
                &config.admin_auth,
            )?;
            for KeyConfig { key, vault } in key_configs {
                KEY_VAULT_MAP.save(deps.storage, key, &deps.api.addr_validate(&vault)?)?;
            }
            Ok(Response::default().add_attribute("action", "register_vaults"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    require_enabled(&config)?;
    pad_query_result(
        match msg {
            QueryMsg::GetPrice { key } => to_binary(&query_price(&deps, &env, &config, key)?),
            QueryMsg::GetPrices { keys } => {
                let mut results = vec![];
                for key in keys {
                    results.push(query_price(&deps, &env, &config, key)?);
                }
                to_binary(&results)
            }
            QueryMsg::GetConfig {} => to_binary(&config),
        },
        BLOCK_SIZE,
    )
}

fn query_price(deps: &Deps, env: &Env, config: &Config, key: String) -> StdResult<OraclePrice> {
    const RATE_SUFFIX: &str = " Rate";

    let is_rate = key.ends_with(RATE_SUFFIX);
    let load_key = if is_rate {
        key[..key.len() - RATE_SUFFIX.len()].to_string()
    } else {
        key.clone()
    };
    let vault_addr = KEY_VAULT_MAP
        .may_load(deps.storage, load_key.clone())
        .map_err(|_| StdError::generic_err("Key not found"))?
        .ok_or_else(|| StdError::generic_err("Key not found"))?;

    let vault = query_vault(&config.money_market, &vault_addr, &deps.querier)?;

    let x_token_info = token_info_query(
        deps.querier,
        255,
        vault.x_token.code_hash,
        vault.x_token.address.to_string(),
    )?;

    let total_supply = match x_token_info.total_supply {
        Some(supply) => supply,
        None => return Err(StdError::generic_err("No total supply")),
    };

    if total_supply.is_zero() {
        return Ok(OraclePrice {
            key,
            data: ReferenceData {
                rate: Uint256::zero(),
                last_updated_base: env.block.time.seconds(),
                last_updated_quote: env.block.time.seconds(),
            },
        });
    }

    let backing_balance = vault.loanable
        + vault.lent_amount
        + (vault.lifetime_interest_owed - vault.lifetime_interest_paid);

    let exchange_rate = (Uint256::from(backing_balance) * Uint256::from(exp10(18).as_u128()))
        / Uint256::from(total_supply);

    if is_rate {
        Ok(OraclePrice {
            key,
            data: ReferenceData {
                rate: exchange_rate,
                last_updated_base: env.block.time.seconds(),
                last_updated_quote: env.block.time.seconds(),
            },
        })
    } else {
        let underlying_price =
            query_router_price(&config.router, &deps.querier, vault.oracle_key.clone())?;
        Ok(OraclePrice {
            key,
            data: ReferenceData {
                rate: exchange_rate * underlying_price.data.rate
                    / Uint256::from(exp10(18).as_u128()),
                last_updated_base: underlying_price.data.last_updated_base,
                last_updated_quote: underlying_price.data.last_updated_quote,
            },
        })
    }
}

pub fn query_router_price(
    router: &Contract,
    querier: &QuerierWrapper,
    key: impl Into<String>,
) -> StdResult<PriceResponse> {
    OracleQuery::GetPrice { key: key.into() }.query(querier, router)
}

pub fn query_vault(
    money_market: &Contract,
    vault: &Addr,
    querier: &QuerierWrapper,
) -> StdResult<money_market::VaultState> {
    money_market::QueryMsg::GetVault {
        token: vault.to_string(),
    }
    .query(querier, money_market)
}
