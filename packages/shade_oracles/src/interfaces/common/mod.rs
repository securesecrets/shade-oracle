use crate::storage::{Item, ItemStorage};
use crate::{BLOCK_SIZE, Query, InstantiateCallback, ExecuteCallback, ResponseStatus, Contract, pad_handle_result, pad_query_result};
use self::querier::verify_admin;
use cosmwasm_schema::{cw_serde};
use cosmwasm_std::{Uint128, Uint256, StdError, QueryResponse, StdResult, DepsMut, MessageInfo, Env, Response, Deps, to_binary, Api};
use shade_protocol::utils::asset::{Dependency, RawDependency, Dependencies, RawContract};

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
pub struct ConfigUpdates {
    pub supported_keys: Option<Vec<String>>,
    pub symbols: Option<Vec<String>>,
    pub dependencies: Option<Vec<RawDependency>>,
    pub router: Option<RawContract>,
    pub only_band: Option<bool>,
    pub enabled: Option<bool>,
}

#[cw_serde]
pub struct InstantiateCommonConfig {
    pub supported_keys: Option<Vec<String>>,
    pub symbols: Option<Vec<String>>,
    pub dependencies: Option<Vec<RawDependency>>,
    pub router: RawContract,
    pub enabled: bool,
    pub only_band: bool,
}

impl InstantiateCommonConfig {
    pub fn new(
        supported_keys: Option<Vec<String>>,
        symbols: Option<Vec<String>>,
        dependencies: Option<Vec<RawDependency>>,
        router: RawContract,
        enabled: bool,
        only_band: bool
    ) -> Self {
        InstantiateCommonConfig { supported_keys, symbols, dependencies, router, enabled, only_band }
    }
    pub fn into_valid(self, api: &dyn Api) -> StdResult<CommonConfig> {
        let dependencies = self.dependencies.unwrap_or_default().iter().map(|d| d.into_valid(api) ).collect::<StdResult<Vec<_>>>()?;
        Ok(CommonConfig {
            supported_keys: self.supported_keys.unwrap_or_default(),
            symbols: self.symbols.unwrap_or_default(), 
            dependencies,
            router: self.router.into_valid(api)?,
            enabled: self.enabled,
            only_band: self.only_band,
        })
    }
}

#[cw_serde]
pub struct CommonConfig {
    pub supported_keys: Vec<String>,
    pub symbols: Vec<String>,
    pub dependencies: Vec<Dependency>,
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

pub mod querier;

#[cfg(feature = "core")]
impl ItemStorage for CommonConfig {
    const ITEM: Item<'static, Self> = Item::new("commonconfig");
}

#[cfg(feature = "core")]
pub fn oracle_exec(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
    oracle: impl Oracle
) -> StdResult<Response> {
    let mut config = CommonConfig::load(deps.storage)?;
    verify_admin(&config.router, &deps.querier, info.sender)?;
    let msg = match msg {
        ExecuteMsg::UpdateConfig { updates } => oracle.try_update_config(deps, env, updates, &mut config),
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
    let config = CommonConfig::load(deps.storage)?;
    let resp = match msg {
        OracleQuery::GetConfig {  } => to_binary(&oracle.config_resp(config)),
        OracleQuery::GetPrice { key } => {
            let supported_keys = config.supported_keys.as_slice();
            if !supported_keys.is_empty() && !supported_keys.contains(&key) {
                return Err(throw_unsupported_symbol_error(key));
            } else {
                return to_binary(&oracle.try_query_price(deps, env, key, config)?);
            }
        },
        OracleQuery::GetPrices { keys } => {
            let supported_keys = config.supported_keys.as_slice();
            let keys_slice = keys.as_slice();
            let mut key = "";
            if !supported_keys.is_empty() && !keys_slice.iter().any(|k| -> bool {
                key = k;
                !supported_keys.contains(k)
            }) {
                return Err(throw_unsupported_symbol_error(key.to_string()));
            } else {
                return to_binary(&oracle.try_query_prices(deps, env, keys, config)?)
            }
        },
    };
    pad_query_result(resp, BLOCK_SIZE)
}

#[cfg(feature = "core")]
pub struct OracleImpl;

#[cfg(feature = "core")]
impl Oracle for OracleImpl {
    fn _try_query_price(&self, _deps: Deps,_env: &Env, _key: String, _config: &CommonConfig) -> StdResult<OraclePrice> {
        Err(StdError::generic_err("Need to be implemented."))
    }
}

#[cfg(feature = "core")]
pub trait Oracle {
    /// Instantiates a CommonConfig from InstantiateCommonConfig, saving it to store.
    fn init_config(
        &self,
        deps: DepsMut,
        config: InstantiateCommonConfig,
    ) -> StdResult<CommonConfig> {
        let config = config.into_valid(deps.api)?;
        config.save(deps.storage)?;
        Ok(config)
    }

    #[allow(clippy::too_many_arguments)]
    fn try_update_config(
        &self,
        deps: DepsMut, 
        _env: Env, 
        updates: ConfigUpdates,
        config: &mut CommonConfig
    ) -> StdResult<Response> {
        config.supported_keys = updates.supported_keys.unwrap_or_else(|| config.supported_keys.clone());
        config.symbols = updates.symbols.unwrap_or_else(|| config.symbols.clone());
        config.only_band = updates.only_band.unwrap_or(config.only_band);
        config.enabled = updates.enabled.unwrap_or(config.enabled);
        if let Some(router) = updates.router {
            config.router = router.into_valid(deps.api)?;
        }
        if let Some(dependencies) = updates.dependencies {
            config.dependencies = dependencies.into_iter().map(|d| d.into_valid(deps.api)).collect::<StdResult<Vec<_>>>()?;
        }

        Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?))
    }

    fn config_resp(&self, config: CommonConfig) -> ConfigResponse {
        ConfigResponse { config }
    }

    /// Wraps internal implementation in proper response struct.
    fn try_query_price(&self, deps: Deps, env: Env, key: String, config: CommonConfig) -> StdResult<PriceResponse> {
        Ok(PriceResponse { price: self._try_query_price(deps, &env, key, &config )? })
    }

    /// Internal implementation of the query price method.
    fn _try_query_price(&self, deps: Deps, env: &Env, key: String, config: &CommonConfig) -> StdResult<OraclePrice>;

    fn try_query_prices(&self, deps: Deps, env: Env, keys: Vec<String>, config: CommonConfig) -> StdResult<PricesResponse> {
        Ok(PricesResponse { prices: self._try_query_prices(deps, env, keys, config)? })
    }

    fn _try_query_prices(&self, deps: Deps, env: Env, keys: Vec<String>, config: CommonConfig) -> StdResult<Vec<OraclePrice>> {
        let mut prices = vec![];
        for key in keys {
            prices.push(self._try_query_price(deps, &env, key, &config)?);
        }
        Ok(prices)
    }
    
}