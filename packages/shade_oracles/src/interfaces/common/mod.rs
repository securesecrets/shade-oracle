use std::cmp::max;

use shade_protocol::{
    utils::{storage::plus::{ItemStorage}, price::get_precision},
    secret_storage_plus::Item,
    utils::generic_response::ResponseStatus,
    utils::{pad_handle_result, pad_query_result, ExecuteCallback, Query},
};
use crate::BLOCK_SIZE;
use self::querier::verify_admin;
use cosmwasm_schema::{cw_serde};
use cosmwasm_std::{Uint128, StdError, QueryResponse, StdResult, DepsMut, MessageInfo, Env, Response, Deps, to_binary, Api, Storage, QuerierWrapper, Timestamp, OverflowError, Uint256, CheckedMultiplyRatioError};
use shade_protocol::utils::asset::{Contract, RawContract};

pub mod querier;

use super::band::ReferenceData;
/// Default Query API for all oracles.
///
/// Every oracle must support these 3 methods in addition to any specific ones it wants to support.
#[cw_serde]
pub enum OracleQuery {
    GetConfig {},
    GetPrice { key: String },
    GetPrices { keys: Vec<String> },
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: CommonConfig
}

#[cw_serde]
pub struct PriceResponse {
    pub price: OraclePrice
}

#[cw_serde]
pub struct PricesResponse {
    pub prices: Vec<OraclePrice>
}

impl Query for OracleQuery {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { 
        updates: ConfigUpdates,
    },
}

#[cw_serde]
/// Config object passed into the updating of an oracle's common config.
/// 
/// supported_keys - (keys which are allowed by oracle, if none listed, then oracle will support all keys).
/// 
/// router - oracle router
/// 
/// enabled - can we use this oracle?
/// 
/// only_band - will this oracle go directly to band rather than through the router?
pub struct ConfigUpdates {
    pub supported_keys: Option<Vec<String>>,
    pub router: Option<RawContract>,
    pub only_band: Option<bool>,
    pub enabled: Option<bool>,
}

#[cw_serde]
/// Config object passed into the instantiation of an oracle.
/// 
/// supported_keys - (keys which are allowed by oracle, if none listed, then oracle will support all keys).
/// 
/// router - oracle router
/// 
/// enabled - can we use this oracle?
/// 
/// only_band - will this oracle go directly to band rather than through the router?
pub struct InstantiateCommonConfig {
    pub supported_keys: Option<Vec<String>>,
    pub router: RawContract,
    pub enabled: bool,
    pub only_band: bool,
}

impl InstantiateCommonConfig {
    pub fn new(
        supported_keys: Option<Vec<String>>,
        router: RawContract,
        enabled: bool,
        only_band: bool
    ) -> Self {
        InstantiateCommonConfig { supported_keys, router, enabled, only_band }
    }
    pub fn into_valid(self, api: &dyn Api) -> StdResult<CommonConfig> {
        Ok(CommonConfig {
            supported_keys: self.supported_keys.unwrap_or_default(),
            router: self.router.into_valid(api)?,
            enabled: self.enabled,
            only_band: self.only_band,
        })
    }
}

/// Config object stored in all oracles.
/// 
/// supported_keys - (keys which are allowed by oracle, if none listed, then oracle will support all keys).
/// 
/// router - oracle router
/// 
/// enabled - can we use this oracle?
/// 
/// only_band - will this oracle go directly to band rather than through the router?
#[cw_serde]
pub struct CommonConfig {
    pub supported_keys: Vec<String>,
    pub router: Contract,
    pub enabled: bool,
    pub only_band: bool,
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

/// Default HandleAnswer for oracles if only ExecuteMsg implemented is UpdateConfig.
#[cw_serde]
pub enum HandleAnswer {
    UpdateConfig { status: ResponseStatus },
}

#[cw_serde]
pub struct OraclePrice {
    pub key: String,
    pub data: ReferenceData,
}

impl OraclePrice {
    pub fn new(key: String, reference_data: ReferenceData) -> Self {
        OraclePrice {
            key,
            data: reference_data,
        }
    }
    pub fn time_since_updated(&self, time: &Timestamp) -> StdResult<u64> {
        let now = time.seconds();
        let base = self.data.last_updated_base;
        let quote = self.data.last_updated_quote;
        let time_since_base = now.checked_sub(base);
        let time_since_quote = now.checked_sub(quote);
        if time_since_base.is_none() {
            return Err(StdError::Overflow { source: OverflowError::new(cosmwasm_std::OverflowOperation::Sub, now, base ) });
        }
        if time_since_quote.is_none() {
            return Err(StdError::Overflow { source: OverflowError::new(cosmwasm_std::OverflowOperation::Sub, now, quote ) });
        }
        let time_since_updated = max(time_since_base.unwrap(), time_since_quote.unwrap());
        Ok(time_since_updated)
    }
    /// Allows us to pass a variable amount of precision decimals in the future
    /// in case our oracles lose their constant decimal precision (currently 18).
    /// Gets the value for some amount using the price.
    pub fn calc_value(&self, amount: Uint256) -> Result<Uint256, CheckedMultiplyRatioError> {
        amount.checked_multiply_ratio(amount, Uint256::from(get_precision(18)))
    }
}

pub fn throw_unsupported_symbol_error(key: String) -> StdError {
   StdError::generic_err(format!("{} is not supported as a key.", key))
}

pub fn is_disabled(enabled: bool) -> StdResult<()> {
    if !enabled {
        Err(StdError::generic_err("Deprecated oracle."))
    } else {
        Ok(())
    }
}

#[cfg(feature = "core")]
impl ItemStorage for CommonConfig {
    const ITEM: Item<'static, Self> = Item::new("commonconfig");
}

#[cfg(feature = "core")]
pub fn oracle_exec(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
    oracle: impl Oracle
) -> StdResult<Response> {
    let mut config = oracle.verify_admin(deps.storage, &deps.querier, info)?;
    let msg = match msg {
        ExecuteMsg::UpdateConfig { updates } => oracle.try_update_config(deps, updates, &mut config),
    };
    pad_handle_result(msg, BLOCK_SIZE)
}

#[cfg(feature = "core")]
pub fn oracle_query(
    deps: Deps,
    env: Env,
    msg: OracleQuery,
    oracle: impl Oracle
) -> StdResult<QueryResponse> {
    let resp = match msg {
        OracleQuery::GetConfig {  } => {
            let config = CommonConfig::load(deps.storage)?;
            to_binary(&oracle.config_resp(config))
        },
        OracleQuery::GetPrice { key } => {
            let config = oracle.can_query_price(deps, &key)?;
            to_binary(&oracle.price_resp(oracle.try_query_price(deps, &env, key, &config)?))
        },
        OracleQuery::GetPrices { keys } => {
            let config = oracle.can_query_prices(deps, keys.as_slice())?;
            to_binary(&oracle.prices_resp(oracle.try_query_prices(deps, &env, keys, &config)?))
        },
    };
    pad_query_result(resp, BLOCK_SIZE)
}

#[cfg(feature = "core")]
pub struct OracleImpl;

#[cfg(feature = "core")]
impl Oracle for OracleImpl {
    fn try_query_price(&self, _deps: Deps,_env: &Env, _key: String, _config: &CommonConfig) -> StdResult<OraclePrice> {
        Err(StdError::generic_err("Need to be implemented."))
    }
}

#[cfg(feature = "core")]
pub trait Oracle {
    /// Instantiates a CommonConfig from InstantiateCommonConfig, saving it to store.
    fn init_config(
        &self,
        storage: &mut dyn Storage,
        api: &dyn Api,
        config: InstantiateCommonConfig,
    ) -> StdResult<CommonConfig> {
        let config = config.into_valid(api)?;
        config.save(storage)?;
        Ok(config)
    }

    /// The first step before resolution of any execute msg for oracles.
    fn verify_admin(
        &self,
        storage: &mut dyn Storage,
        querier: &QuerierWrapper,
        info: MessageInfo
    ) -> StdResult<CommonConfig> {
        let config = CommonConfig::load(storage)?;
        verify_admin(&config.router, querier, info.sender)?;
        Ok(config)
    }

    #[allow(clippy::too_many_arguments)]
    fn try_update_config(
        &self,
        deps: DepsMut, 
        updates: ConfigUpdates,
        config: &mut CommonConfig
    ) -> StdResult<Response> {
        config.supported_keys = updates.supported_keys.unwrap_or_else(|| config.supported_keys.clone());
        config.only_band = updates.only_band.unwrap_or(config.only_band);
        config.enabled = updates.enabled.unwrap_or(config.enabled);
        if let Some(router) = updates.router {
            config.router = router.into_valid(deps.api)?;
        }

        config.save(deps.storage)?;

        Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?))
    }

    fn config_resp(&self, config: CommonConfig) -> ConfigResponse {
        ConfigResponse { config }
    }

    fn price_resp(&self, price: OraclePrice) -> PriceResponse {
        PriceResponse { price }
    }

    fn prices_resp(&self, prices: Vec<OraclePrice>) -> PricesResponse {
        PricesResponse { prices }
    }

    /// Internal implementation of the query price method.
    fn try_query_price(&self, deps: Deps, env: &Env, key: String, config: &CommonConfig) -> StdResult<OraclePrice>;

    /// Checks if user can query for prices
    fn can_query_prices(&self, deps: Deps, keys: &[String]) -> StdResult<CommonConfig> {
        let config = CommonConfig::load(deps.storage)?;
        is_disabled(config.enabled)?;
        let supported_keys = config.supported_keys.as_slice();
        let mut key = "";
        if !supported_keys.is_empty() && !keys.iter().any(|k| -> bool {
            key = k;
            !supported_keys.contains(k)
        }) {
            return Err(throw_unsupported_symbol_error(key.to_string()));
        }
        Ok(config)
    }

    fn can_query_price(&self, deps: Deps, key: &String) -> StdResult<CommonConfig> {
        let config = CommonConfig::load(deps.storage)?;
        is_disabled(config.enabled)?;
        let supported_keys = config.supported_keys.as_slice();
        if !supported_keys.is_empty() && !supported_keys.contains(key) {
            return Err(throw_unsupported_symbol_error(key.to_string()));
        }
        Ok(config)
    }

    fn try_query_prices(&self, deps: Deps, env: &Env, keys: Vec<String>, config: &CommonConfig) -> StdResult<Vec<OraclePrice>> {
        let mut prices = vec![];
        for key in keys {
            prices.push(self.try_query_price(deps, env, key, config)?);
        }
        Ok(prices)
    }
    
}