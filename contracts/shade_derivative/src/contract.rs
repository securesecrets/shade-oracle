use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, StdError, Storage, Uint128};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use shade_oracles::core::{pad_query_result, ResponseStatus};
use shade_oracles::interfaces::common::OraclePrice;
use shade_oracles::interfaces::providers::{
    mock::{BandExecuteMsg, BandInstantiateMsg, BandMockPrice, Config, ExecuteAnswer},
    BandQueryMsg, ReferenceData,
};
use shade_oracles::ssp::{Item, Map};
use shade_oracles::BLOCK_SIZE;

#[cw_serde]
pub struct Config {
    router: Contract,
    dshd: Contract,
    admin_auth: Contract,
    enabled: bool,
}

#[cw_serde]
pub struct InstantiateMsg {
    router: RawContract,
    dshd: RawContract,
    admin_auth: RawContract,
}
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        router: Option<RawContract>,
        dshd: Option<RawContract>,
        admin_auth: Option<RawContract>,
        enabled: Option<bool>,
    }
}

#[cw_serde]
pub enum QueryMsg {
    GetPrice {
        key: String,
    },
    GetPrices {
        keys: Vec<String>,
    },
    GetConfig: {},
}

#[cw_serde]
pub enum DShdQueryMsg {
    StakingInfo { }
}

pub struct StakingInfoResponse { 
    unbonding_time: Uint128,
    bonded_shd: Uint128,
    available_shd: Uint128,
    rewards: Uint128,
    total_derivative_token_supply: Uint128,
    price: Uint128,
}

pub const UNDERLYING_KEY = "SHD";
pub const PRICE_KEY = "Shade Derivative";
pub const RATE_KEY = "Shade Derivative Rate";

const CONFIG: Item<Config> = Item::new("config");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: BandInstantiateMsg,
) -> StdResult<Response> {
    let now = env.block.time.seconds();

    let admin_auth = msg.admin_auth.validate(deps.api)?;
    let router = msg.router.validate(deps.api)?;
    let dshd = msg.dshd.validate(deps.api)?;

    let config = Config {
        admin_auth,
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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    match msg {
        ExecuteMsg::UpdateConfig => {
            let resp = Response::default();
            config.require_admin(&deps.querier, info.sender)?;
            if let router = Some(msg.router) {
                config.router = router.validate(deps.api)?;
                resp.add_attribute("router", router.address);
            }
            if let admin_auth = Some(msg.admin_auth) {
                config.admin_auth = admin_auth.validate(deps.api)?;
                resp.add_attribute("admin_auth", admin_auth.address);
            }
            if let dshd = Some(msg.dshd) {
                config.dshd = dshd.unwrap().validate(deps.api)?;
                resp.add_attribute("dshd", dshd.address);
            }
            if let enabled = Some(msg.enabled) {
                config.enabled = enabled;
                resp.add_attribute("enabled", enabled);
            }
            CONFIG.save(deps.storage, &config)?;
            Ok(resp.add_attribute("action", "update_config"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: BandQueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    pad_query_result(
        match msg {
            QueryMsg::GetPrice { key } => {
                Ok(OraclePrice::new(key, {
                    rate: query_by_key(
                          deps,
                          key,
                          config.router,
                          config.dshd,
                      )?,
                    last_updated_base: env.block.time.seconds,
                    last_updated_quote: env.block.time.seconds,
                }))
            },
            QueryMsg::GetPrices { keys } => {
            },
        },
        BLOCK_SIZE,
    )
}

fn query_oracle_price_by_key(
    deps: Deps,
    key: String,
    router: Contract,
    dshd: Contract,
) -> StdResult<Binary> {

    if key == PRICE_KEY {
        query_price(deps, router)
    }
    else if key == RATE_KEY {
        query_rate(deps, router)
    }
}

fn query_price(
    deps: Deps,
    router: Contract,
) -> StdResult<Uint128> {
    let underlying_price = query_router_price(router, deps.querier, UNDERLYING_KEY)?.rate;
    let rate = query_rate(deps, dshd)?;
    Ok((underlying_price * rate) / exp10(18))
}

fn query_rate(
    deps: Deps,
    dshd: Contract,
) -> StdResult<Uint128> {
    let staking_info = query_staking_info(dshd, deps.querier)?;
    // normalize from 10^6 to 10^18
    Ok(staking_info.price * exp10(12))
}

pub fn query_router_price(
    oracle: &Contract,
    querier: &QuerierWrapper,
    key: impl Into<String>,
) -> StdResult<PriceResponse> {
    OracleQuery::GetPrice { key: key.into() }.query(querier, oracle)
}

pub fn query_staking_info(
    dshd: &Contract,
    querier: &QuerierWrapper,
) -> StdResult<StakingInfoResponse> {
    DShdQueryMsg::StakingInfo { }.query(querier, dshd)
}
