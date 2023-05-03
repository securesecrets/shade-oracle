//! Staking derivative oracles calculate the price of 1 staking derivative token
//! based off its exchange rate with its underlying token.
//! Requires a bot with some permission to push the rates.

use super::*;

pub mod msg {
    use crate::interfaces::common::{config::CommonConfigResponse, PriceResponse, PricesResponse};
    use cosmwasm_std::Decimal256;

    use super::*;

    impl_msg_callbacks!();

    #[cw_serde]
    pub struct InstantiateMsg {
        pub router: RawContract,
    }

    #[cw_serde]
    pub enum ExecuteMsg {
        /// For setting and updating derivative data config.
        SetDerivatives(Vec<RawDerivativeData>),
        RemoveDerivatives(Vec<String>),
        UpdateConfig(RawContract),
        UpdateDerivatives(DerivativeUpdates),
        SetStatus(bool),
    }

    #[cw_serde]
    pub enum DerivativeUpdates {
        Rates(Vec<(String, Decimal256)>),
        Config(Vec<(String, DerivativeDataConfigUpdate)>),
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {
        #[returns(PriceResponse)]
        GetPrice { key: String },
        #[returns(PricesResponse)]
        GetPrices { keys: Vec<String> },
        #[returns(CommonConfigResponse)]
        GetConfig {},
        #[returns(DerivativesResponse)]
        GetDerivatives {},
    }

    pub type DerivativesResponse = Vec<DerivativeData>;

    #[cw_serde]
    pub struct RawDerivativeData {
        /// The key for this derivative.
        pub key: String,
        /// The key for the asset the derivative is for.
        pub underlying_key: String,
        pub initial_rate: Decimal256,
        pub rate_timeout: u64,
        pub rate_max_change: Decimal256,
    }

    #[cw_serde]
    pub struct DerivativeDataConfigUpdate {
        pub underlying_key: Option<String>,
        pub rate_timeout: Option<u64>,
        pub rate_max_change: Option<Decimal256>,
    }

    #[cw_serde]
    pub struct DerivativeData {
        pub key: String,
        pub underlying_key: String,
        pub rate: DerivativeRate,
    }

    #[cw_serde]
    pub struct DerivativeRate {
        pub value: Decimal256,
        pub timeout: u64,
        pub last_updated: u64,
        pub max_change: Decimal256,
    }

    impl DerivativeDataConfigUpdate {
        pub fn new(
            underlying_key: Option<String>,
            rate_timeout: Option<u64>,
            rate_max_change: Option<Decimal256>,
        ) -> Self {
            Self {
                underlying_key,
                rate_max_change,
                rate_timeout,
            }
        }
    }
}

#[cfg(feature = "derivatives")]
pub use msg::*;
#[cfg(feature = "derivatives")]
pub use state::*;
#[cfg(feature = "derivatives")]
mod state {
    use std::collections::HashMap;

    use super::*;
    use crate::{interfaces::{
        common::{
            config::{CommonConfig, CommonConfigResponse},
            OraclePrice, PriceResponse, PricesResponse,
        },
        providers::ReferenceData,
        querier::{query_price as query_router_price, query_prices as query_router_prices},
    }, math::GeneralMath};
    use cosmwasm_std::{Decimal256, Env, StdResult, Storage, QueryResponse, Deps, to_binary, Response, DepsMut};
    use shade_protocol::utils::{pad_query_result, pad_handle_result};
    use ssp::{Item, ItemStorage, Map};

    impl DerivativeRate {
        pub fn new(
            value: Decimal256,
            timeout: u64,
            last_updated: u64,
            max_change: Decimal256,
        ) -> StdResult<Self> {
            let mut rate = Self {
                value,
                timeout,
                last_updated,
                max_change,
            };
            rate.set_value(value)?;
            Ok(rate)
        }
        pub fn set_value(&mut self, value: Decimal256) -> StdResult<()> {
            if value.is_zero() {
                return Err(StdError::generic_err("Rate must be greater than 0"));
            };
            self.value = value;
            Ok(())
        }
        pub fn require_fresh(&self, now: u64) -> StdResult<()> {
            if now - self.last_updated > self.timeout {
                Err(StdError::generic_err(format!(
                    "Derivative rate is stale. Last updated {0}. Current time {now}.",
                    self.last_updated
                )))
            } else {
                Ok(())
            }
        }
    }

    impl DerivativeData {
        pub fn new(
            key: String,
            underlying_key: String,
            initial_rate: Decimal256,
            rate_max_change: Decimal256,
            rate_timeout: u64,
            last_updated: u64,
        ) -> StdResult<Self> {
            let rate =
                DerivativeRate::new(initial_rate, rate_timeout, last_updated, rate_max_change)?;
            Ok(Self {
                key,
                underlying_key,
                rate,
            })
        }
    }

    #[cw_serde]
    pub struct BotStakingDerivativesOracle {
        pub config: CommonConfig,
    }

    impl ItemStorage for BotStakingDerivativesOracle {
        const ITEM: Item<'static, Self> = Item::new("staking_derivatives_oracle");
    }

    impl<'a> BotStakingDerivativesOracle {
        // Keyed by its symbol.
        pub const DERIVATIVES: Map<'static, &'a str, DerivativeData> = Map::new("derivatives");
    }

    impl BotStakingDerivativesOracle {
        /// To be appended to key to signal that consumer wants the rate.
        pub const RATE_STRING: &'static str = " Rate";

        pub fn add_plaintext_attribute(
            response: Response,
            prefix: &'static str,
            action: &'static str,   
        ) -> Response {
            response.add_attribute_plaintext("action", format!("{}{}", prefix, action))
        }

        pub fn instantiate(
            deps: DepsMut,
            msg: InstantiateMsg,
        ) -> StdResult<Response> {
            let config = CommonConfig::init(deps.api, deps.storage, msg.router)?;
            BotStakingDerivativesOracle { config }.save(deps.storage)?;
            Ok(Response::new().add_attribute("action", "instantiate"))
        }

        pub fn execute(
            deps: DepsMut,
            env: Env,
            info: MessageInfo,
            msg: ExecuteMsg,
            rate_permission: impl ToString,
            prefix: &'static str,
        ) -> StdResult<Response> {
            let mut oracle = BotStakingDerivativesOracle::load(deps.storage)?;
            let resp = Response::new();
            let resp = match msg {
                ExecuteMsg::SetStatus(status) => {
                    oracle.config.require_admin(&deps.querier, info)?;
                    oracle.config.update_config(deps.api, Some(status), None)?;
                    oracle.save(deps.storage)?;
                    Self::add_plaintext_attribute(resp, prefix, "set_status")
                }
                _ => {
                    oracle.config.require_enabled()?;
                    let now = env.block.time.seconds();
                    match msg {
                        ExecuteMsg::SetDerivatives(data) => {
                            oracle.config.require_admin(&deps.querier, info)?;
                            oracle.set_derivatives(deps.storage, now, data)?;
                            Self::add_plaintext_attribute(resp, prefix, "set_derivatives")
                        }
                        ExecuteMsg::RemoveDerivatives(keys) => {
                            oracle.config.require_admin(&deps.querier, info)?;
                            BotStakingDerivativesOracle::remove_keys(deps.storage, keys)?;
                            Self::add_plaintext_attribute(resp, prefix, "remove_derivatives")
                        }
                        ExecuteMsg::UpdateDerivatives(update) => match update {
                            DerivativeUpdates::Rates(rates) => {
                                oracle.config.require_permission(
                                    &deps.querier,
                                    &info.sender,
                                    rate_permission,
                                )?;
                                BotStakingDerivativesOracle::update_rates(deps.storage, now, rates)?;
                                Self::add_plaintext_attribute(resp, prefix, "update_derivative_rates")
                            }
                            DerivativeUpdates::Config(configs) => {
                                oracle.config.require_admin(&deps.querier, info)?;
                                oracle.update_derivatives(deps.storage, configs)?;                               Self::add_plaintext_attribute(resp, prefix, "update_derivative_configs")
                            }
                        },
                        ExecuteMsg::UpdateConfig(new_router) => {
                            oracle.config.require_admin(&deps.querier, info)?;
                            oracle
                                .config
                                .update_config(deps.api, None, Some(new_router))?;
                            oracle.save(deps.storage)?;
                            Self::add_plaintext_attribute(resp, prefix, "router_update")
                        }
                        _ => panic!("Code should never go here."),
                    }
                }
            };
            pad_handle_result(Ok(resp), BLOCK_SIZE)
        }

        pub fn query(
            deps: Deps,
            env: Env,
            msg: QueryMsg
        ) -> StdResult<QueryResponse> {
            let oracle = Self::load(deps.storage)?;
            pad_query_result(
                match msg {
                    QueryMsg::GetPrice { key } => {
                        oracle.config.require_enabled()?;
                        to_binary(&oracle.query_price(
                            &env,
                            deps.storage,
                            &deps.querier,
                            key,
                        )?)
                    }
                    QueryMsg::GetPrices { keys } => {
                        oracle.config.require_enabled()?;
                        to_binary(&oracle.query_prices(
                            &env,
                            deps.storage,
                            &deps.querier,
                            keys,
                        )?)
                    }
                    QueryMsg::GetConfig {} => to_binary(&oracle.query_config(deps.storage)?),
                    QueryMsg::GetDerivatives {} => to_binary(&Self::get_supported_derivatives(deps.storage)?),
                },
                BLOCK_SIZE,
            )
        }

        pub fn create_rate_key(key: &str) -> String {
            format!("{}{}", key, Self::RATE_STRING)
        }

        pub fn process_key(key: &str) -> (bool, String) {
            let mut is_rate = false;
            let mut processed_key = key;
            if key.ends_with(Self::RATE_STRING) {
                processed_key = key.trim_end_matches(Self::RATE_STRING);
                is_rate = true;
            }
            (is_rate, processed_key.to_string())
        }

        pub fn set_derivatives(
            &self,
            storage: &mut dyn Storage,
            now: u64,
            derivatives: Vec<RawDerivativeData>,
        ) -> StdResult<()> {
            for data in derivatives {
                let data = DerivativeData::new(
                    data.key,
                    data.underlying_key,
                    data.initial_rate,
                    data.rate_max_change,
                    data.rate_timeout,
                    now,
                )?;
                Self::DERIVATIVES.save(storage, &data.key, &data)?;
                CommonConfig::add_supported_key(storage, &data.key)?;
            }
            Ok(())
        }

        pub fn remove_keys(storage: &mut dyn Storage, keys: Vec<String>) -> StdResult<()> {
            let mut supported_keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
            for key in keys {
                if supported_keys.remove(&key) {
                    Self::DERIVATIVES.remove(storage, &key);
                }
            }
            CommonConfig::SUPPORTED_KEYS.save(storage, &supported_keys)?;
            Ok(())
        }

        pub fn get_derivative_data_resp(
            key: &String,
            storage: &dyn Storage,
        ) -> StdResult<DerivativeData> {
            let data = Self::DERIVATIVES.load(storage, key)?;
            Ok(data)
        }

        pub fn update_derivatives(
            &self,
            storage: &mut dyn Storage,
            updates: Vec<(String, DerivativeDataConfigUpdate)>,
        ) -> StdResult<()> {
            for (key, update) in updates {
                let mut data = Self::DERIVATIVES.load(storage, &key)?;
                if let Some(underlying_key) = update.underlying_key {
                    data.underlying_key = underlying_key;
                }
                if let Some(rate_timeout) = update.rate_timeout {
                    data.rate.timeout = rate_timeout;
                }
                Self::DERIVATIVES.save(storage, &key, &data)?;
            }
            Ok(())
        }

        pub fn update_rates(
            storage: &mut dyn Storage,
            now: u64,
            rates: Vec<(String, Decimal256)>,
        ) -> StdResult<()> {
            for (key, rate) in rates {
                let mut data = Self::DERIVATIVES.load(storage, &key)?;
                GeneralMath::require_within_precision(rate, data.rate.value, data.rate.max_change)?;
                data.rate.last_updated = now;
                data.rate.value = rate;
                Self::DERIVATIVES.save(storage, &key, &data)?;
            }
            Ok(())
        }

        pub fn get_supported_derivatives(storage: &dyn Storage) -> StdResult<DerivativesResponse> {
            let keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
            let mut supported_pairs = vec![];
            for key in keys {
                supported_pairs.push(BotStakingDerivativesOracle::get_derivative_data_resp(
                    &key, storage,
                )?);
            }
            Ok(supported_pairs)
        }

        /// If price of underlying is provided, assume we want the price of the derivative, else
        /// assume we want the derivative rate itself.
        fn calculate_price(
            env: &Env,
            data: &DerivativeData,
            underlying_price: Option<&OraclePrice>,
        ) -> StdResult<ReferenceData> {
            let now = env.block.time.seconds();
            data.rate.require_fresh(now)?;
            match underlying_price {
                Some(price) => {
                    let rate = data.rate.value * price.data().rate;
                    Ok(ReferenceData::new(
                        rate,
                        price.data().last_updated_base,
                        price.data().last_updated_quote,
                    ))
                },
                None => {
                    Ok(ReferenceData::new(
                        data.rate.value.atomics(),
                        data.rate.last_updated,
                        data.rate.last_updated,
                    ))
                },
            }
        }

        fn query_price(
            &self,
            env: &Env,
            storage: &dyn Storage,
            querier: &QuerierWrapper,
            key: String,
        ) -> StdResult<PriceResponse> {
            let original_key = key.clone();
            let (is_rate, key) = Self::process_key(&key);
            let stored_data = Self::DERIVATIVES.load(storage, &key)?;

            let underlying_price = if is_rate {
                None
            } else {
                Some(query_router_price(&self.config.router, querier, &stored_data.underlying_key)?)
            };
            
            let data = Self::calculate_price(env, &stored_data, underlying_price.as_ref())?;
            
            Ok(OraclePrice::new(original_key, data))
        }

        fn query_prices(
            &self,
            env: &Env,
            storage: &dyn Storage,
            querier: &QuerierWrapper,
            keys: Vec<String>,
        ) -> StdResult<PricesResponse> {
            // Preserve symbol order
            let ordered_prices: Vec<OraclePrice> = keys
            .iter()
            .map(|key| {    
                OraclePrice::new(key.clone(), ReferenceData::default())
            })
            .collect();
            // Temp vector of fetched prices
            let mut derivative_data_cache: HashMap<String, DerivativeData> = HashMap::new();
            let mut underlying_price_symbols: Vec<String> = vec![];

            for key in keys.iter() {
                let (is_rate, key) = Self::process_key(key);
                if !derivative_data_cache.contains_key(&key) {
                    let data = Self::DERIVATIVES.load(storage, &key)?;
                    derivative_data_cache.insert(key.clone(), data);
                }
                if let Some(data) = derivative_data_cache.get(&key) {
                    if !is_rate {
                        let underlying_key = &data.underlying_key;
                        if !underlying_price_symbols.contains(underlying_key) {
                            underlying_price_symbols.push(underlying_key.clone());
                        }
                    }
                }
            }

            let underlying_prices = query_router_prices(
                &self.config.router,
                querier,
                &underlying_price_symbols,
            )?;

            let mut underlying_price_cache: HashMap<String, OraclePrice> = HashMap::new();

            for price in underlying_prices {
                underlying_price_cache.insert(price.key().clone(), price);
            }

            // For every fetched price, find its position in the original request and replace the placeholder data with the actual data for that symbol.
            let ordered_prices: StdResult<Vec<OraclePrice>> = ordered_prices
                .into_iter()
                .map(|mut p| -> StdResult<OraclePrice> {
                    let (is_rate, key) = Self::process_key(p.key());
                    let stored_data = derivative_data_cache.get(&key).unwrap();
                    let underlying_price = if is_rate {
                        None
                    } else {
                        Some(underlying_price_cache.get(&stored_data.underlying_key).unwrap())
                    };
                    let data = Self::calculate_price(env, stored_data, underlying_price)?;
                    p.data = data;
                    Ok(p)
                })
                .collect();
            Ok(ordered_prices?)
        }

        fn query_config(&self, storage: &dyn Storage) -> StdResult<CommonConfigResponse> {
            let mut resp = self.config.get_resp(storage)?;
            let mut supported_keys = vec![];
            for key in &resp.supported_keys {
                supported_keys.push(key.to_string());
                supported_keys.push(Self::create_rate_key(key));
            }
            resp.supported_keys = supported_keys;
            Ok(resp)
        }
    }
}
