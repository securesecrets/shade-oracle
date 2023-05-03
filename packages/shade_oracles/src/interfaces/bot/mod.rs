//! Configurable oracle that allows for an entity with X permission to push Y prices.
use super::*;

pub mod msg {
    use crate::interfaces::common::{config::CommonConfigResponse, PriceResponse, PricesResponse};
    use cosmwasm_std::{Decimal256};

    use super::*;

    impl_msg_callbacks!();

    #[cw_serde]
    pub struct InstantiateMsg {
        pub router: RawContract,
    }

    #[cw_serde]
    pub enum ExecuteMsg {
        SetRates(Vec<RawRateData>),
        RemoveRates(Vec<String>),
        UpdateConfig(RawContract),
        UpdateRates(RateDataUpdate),
        SetStatus(bool),
    }

    #[cw_serde]
    pub enum RateDataUpdate {
        Rates(Vec<(String, Decimal256)>),
        Config(Vec<(String, RateDataConfigUpdate)>),
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
        #[returns(RatesResponse)]
        GetRates {},
    }

    pub type RatesResponse = Vec<RateData>;

    #[cw_serde]
    pub struct RawRateData {
        /// The key for this rate.
        pub key: String,
        /// Some initial rate.
        pub initial_rate: Decimal256,
        /// When this rate should be considered stale.
        pub timeout: u64,
        /// The maximum % change allowed for this rate per update.
        pub max_change: Decimal256,
        /// Permission required to update this rate.
        pub permission: String,
    }

    #[cw_serde]
    pub struct RateData {
        pub key: String,
        pub rate: Decimal256,
        pub last_updated: u64,
        pub timeout: u64,
        pub max_change: Decimal256,
        pub permission: String,
    }

    #[cw_serde]
    pub struct RateDataConfigUpdate {
        pub timeout: Option<u64>,
        pub max_change: Option<Decimal256>,
    }

    impl RateDataConfigUpdate {
        pub fn new(
            timeout: Option<u64>,
            max_change: Option<Decimal256>,
        ) -> Self {
            Self {
                timeout,
                max_change,
            }
        }
    }
}

#[cfg(feature = "bot")]
pub use msg::*;
#[cfg(feature = "bot")]
pub use state::*;
#[cfg(feature = "bot")]
mod state {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use crate::{interfaces::{
        common::{
            config::{CommonConfig, CommonConfigResponse},
            OraclePrice, PriceResponse, PricesResponse,
        },
        providers::ReferenceData,
    }, math::GeneralMath};
    use cosmwasm_std::{Decimal256, Env, StdResult, Storage, QueryResponse, Deps, to_binary, Response, DepsMut};
    use shade_protocol::utils::{pad_query_result, pad_handle_result};
    use ssp::{Item, ItemStorage, Map};

    #[cw_serde]
    struct Data {
        pub rate: Decimal256,
        pub last_updated: u64,
        pub timeout: u64,
        pub max_change: Decimal256,
        pub permission: String,
    }

    impl Data {
        pub fn new(
            value: Decimal256,
            timeout: u64,
            last_updated: u64,
            max_change: Decimal256,
            permission: String,
        ) -> StdResult<Self> {
            let mut rate = Self {
                timeout,
                last_updated,
                max_change,
                rate: Decimal256::zero(),
                permission,
            };
            rate.set_value(value)?;
            Ok(rate)
        }
        pub fn set_value(&mut self, value: Decimal256) -> StdResult<()> {
            if value.is_zero() {
                return Err(StdError::generic_err("Rate must be greater than 0"));
            };
            self.rate = value;
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

    impl RateData {
        pub fn new(
            key: String,
            rate: Decimal256,
            last_updated: u64,
            timeout: u64,
            max_change: Decimal256,
            permission: String,
        ) -> Self {
            Self {
                key,
                rate,
                last_updated,
                timeout,
                max_change,
                permission,
            }
        }
    }

    #[cw_serde]
    pub struct BotRateOracle {
        pub config: CommonConfig,
    }

    impl ItemStorage for BotRateOracle {
        const ITEM: Item<'static, Self> = Item::new("bot_rates_oracle");
    }

    impl<'a> BotRateOracle {
        // Keyed by its symbol.
        const RATES: Map<'static, &'a str, Data> = Map::new("rates");
    }

    impl BotRateOracle {
        fn add_plaintext_attribute(
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
            BotRateOracle { config }.save(deps.storage)?;
            Ok(Response::new().add_attribute("action", "instantiate"))
        }

        pub fn execute(
            deps: DepsMut,
            env: Env,
            info: MessageInfo,
            msg: ExecuteMsg,
            prefix: &'static str,
        ) -> StdResult<Response> {
            let mut oracle = BotRateOracle::load(deps.storage)?;
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
                        ExecuteMsg::SetRates(data) => {
                            oracle.config.require_admin(&deps.querier, info)?;
                            oracle.set_rates(deps.storage, now, data)?;
                            Self::add_plaintext_attribute(resp, prefix, "set_rates")
                        }
                        ExecuteMsg::RemoveRates(keys) => {
                            oracle.config.require_admin(&deps.querier, info)?;
                            BotRateOracle::remove_rates(deps.storage, keys)?;
                            Self::add_plaintext_attribute(resp, prefix, "remove_rates")
                        }
                        ExecuteMsg::UpdateRates(update) => match update {
                            RateDataUpdate::Rates(rates) => {
                                oracle.update_rates(&deps.querier, &info.sender, deps.storage, now, rates)?;
                                Self::add_plaintext_attribute(resp, prefix, "update_rates")
                            }
                            RateDataUpdate::Config(configs) => {
                                oracle.config.require_admin(&deps.querier, info)?;
                                oracle.update_rate_configs(deps.storage, configs)?;                               Self::add_plaintext_attribute(resp, prefix, "update_rate_configs")
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
                            key,
                        )?)
                    }
                    QueryMsg::GetPrices { keys } => {
                        oracle.config.require_enabled()?;
                        to_binary(&oracle.query_prices(
                            &env,
                            deps.storage,
                            keys,
                        )?)
                    }
                    QueryMsg::GetConfig {} => to_binary(&oracle.query_config(deps.storage)?),
                    QueryMsg::GetRates {} => to_binary(&Self::get_supported_rates(deps.storage)?),
                },
                BLOCK_SIZE,
            )
        }

        fn set_rates(
            &self,
            storage: &mut dyn Storage,
            now: u64,
            rates: Vec<RawRateData>,
        ) -> StdResult<()> {
            for raw_data in rates {
                let data = Data::new(
                    raw_data.initial_rate,
                    raw_data.timeout,
                    now,
                    raw_data.max_change,
                    raw_data.permission,
                )?;
                Self::RATES.save(storage, &raw_data.key, &data)?;
                CommonConfig::add_supported_key(storage, &raw_data.key)?;
            }
            Ok(())
        }

        fn remove_rates(storage: &mut dyn Storage, keys: Vec<String>) -> StdResult<()> {
            let mut supported_keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
            for key in keys {
                if supported_keys.remove(&key) {
                    Self::RATES.remove(storage, &key);
                }
            }
            CommonConfig::SUPPORTED_KEYS.save(storage, &supported_keys)?;
            Ok(())
        }

        fn update_rate_configs(
            &self,
            storage: &mut dyn Storage,
            updates: Vec<(String, RateDataConfigUpdate)>,
        ) -> StdResult<()> {
            for (key, update) in updates {
                let mut data = Self::RATES.load(storage, &key)?;
                if let Some(max_change) = update.max_change {
                    data.max_change = max_change;
                }
                if let Some(timeout) = update.timeout {
                    data.timeout = timeout;
                }
                Self::RATES.save(storage, &key, &data)?;
            }
            Ok(())
        }

        fn update_rates(
            &self,
            querier: &QuerierWrapper,
            sender: &Addr,
            storage: &mut dyn Storage,
            now: u64,
            rates: Vec<(String, Decimal256)>,
        ) -> StdResult<()> {
            let mut required_permissions: HashSet<String> = HashSet::new();
            for (key, rate) in rates {
                let mut data = Self::RATES.load(storage, &key)?;
                required_permissions.insert(data.permission.clone());
                GeneralMath::require_within_precision(rate, data.rate, data.max_change)?;
                data.last_updated = now;
                data.rate = rate;
                Self::RATES.save(storage, &key, &data)?;
            }
            // Check that the sender has all the required permissions before allowing the transaction to succeed.
            for permission in required_permissions {
                self.config.require_permission(querier, sender, &permission)?;
            }
            Ok(())
        }

        fn get_rate_data(
            key: &str,
            storage: &dyn Storage,
        ) -> StdResult<RateData> {
            let data = Self::RATES.load(storage, key)?;
            Ok(RateData::new(
                key.to_string(),
                data.rate,
                data.last_updated,
                data.timeout,
                data.max_change,
                data.permission,
            ))
        }

        fn get_supported_rates(storage: &dyn Storage) -> StdResult<RatesResponse> {
            let keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
            let mut supported_pairs = vec![];
            for key in keys {
                supported_pairs.push(BotRateOracle::get_rate_data(
                    &key, storage,
                )?);
            }
            Ok(supported_pairs)
        }

        fn query_price(
            &self,
            env: &Env,
            storage: &dyn Storage,
            key: String,
        ) -> StdResult<PriceResponse> {
            let rate = Self::RATES.load(storage, &key)?;
            rate.require_fresh(env.block.time.seconds())?;
            Ok(OraclePrice::new(key, ReferenceData { rate: rate.rate.atomics(), last_updated_base: rate.last_updated, last_updated_quote: rate.last_updated }))
        }

        fn query_prices(
            &self,
            env: &Env,
            storage: &dyn Storage,
            keys: Vec<String>,
        ) -> StdResult<PricesResponse> {
            // Preserve symbol order
            let mut unique_keys: HashSet<String> = HashSet::new();
            let ordered_prices: Vec<OraclePrice> = keys
            .iter()
            .map(|key| {    
                unique_keys.insert(key.clone());
                OraclePrice::new(key.clone(), ReferenceData::default())
            })
            .collect();

            let mut price_cache: HashMap<String, OraclePrice> = HashMap::new();

            for key in unique_keys {
                if !price_cache.contains_key(key.as_str()) {
                    price_cache.insert(key.clone(), self.query_price(env, storage, key)?);
                }
            }

            // For every fetched price, find its position in the original request and replace the placeholder data with the actual data for that symbol.
            let ordered_prices: StdResult<Vec<OraclePrice>> = ordered_prices
                .into_iter()
                .map(|mut p| -> StdResult<OraclePrice> {
                    if let Some(data) = price_cache.get(&p.key) {
                        p.data = data.data.clone();
                    }
                    Ok(p)
                })
                .collect();
            ordered_prices
        }

        fn query_config(&self, storage: &dyn Storage) -> StdResult<CommonConfigResponse> {
            let mut resp = self.config.get_resp(storage)?;
            let mut supported_keys = vec![];
            for key in &resp.supported_keys {
                supported_keys.push(key.to_string());
            }
            resp.supported_keys = supported_keys;
            Ok(resp)
        }
    }
}
