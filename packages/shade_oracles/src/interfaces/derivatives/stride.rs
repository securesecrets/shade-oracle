//! Staking derivative oracles calculate the price of 1 staking derivative token
//! based off its exchange rate with its underlying token.

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

    #[cw_serde]
    pub enum BotPermission {
        UpdateRates,
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

    impl ToString for BotPermission {
        fn to_string(&self) -> String {
            match self {
                BotPermission::UpdateRates => "SHADE_ORACLES_STRIDE_RATES_BOT".to_string(),
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
    use super::*;
    use crate::interfaces::common::config::CommonConfig;
    use cosmwasm_std::{Decimal256, StdResult, Storage};
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
        pub fn require_valid_change(
            &self,
            now: u64,
            new_rate: Decimal256,
        ) -> StdResult<()> {
            if self.value.eq(&new_rate) {
                return Ok(());
            }

            let change = if self.value > new_rate {
                self.value - new_rate
            } else {
                new_rate - self.value
            };

            if change > self.max_change {
                Err(StdError::generic_err(format!(
                    "Derivative rate is changing too much. Maximum change is {}. Attempted change is {change}.", self.max_change
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
            let rate = DerivativeRate::new(
                initial_rate,
                rate_timeout,
                last_updated,
                rate_max_change,
            )?;
            Ok(Self {
                key,
                underlying_key,
                rate,
            })
        }
    }

    #[cw_serde]
    pub struct StrideStakingDerivativesOracle {
        pub config: CommonConfig,
    }

    impl ItemStorage for StrideStakingDerivativesOracle {
        const ITEM: Item<'static, Self> = Item::new("staking_derivatives_oracle");
    }

    impl<'a> StrideStakingDerivativesOracle {
        // Keyed by its symbol.
        pub const DERIVATIVES: Map<'static, &'a str, DerivativeData> = Map::new("derivatives");
    }

    impl StrideStakingDerivativesOracle {
        /// To be appended to key to signal that consumer wants the rate.
        pub const RATE_STRING: &'static str = " Rate";

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
            querier: &QuerierWrapper,
            now: u64,
            derivatives: Vec<RawDerivativeData>,
        ) -> StdResult<()> {
            for data in derivatives {
                self.config
                    .require_valid_router_symbol(querier, &data.underlying_key)?;
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
            querier: &QuerierWrapper,
            storage: &mut dyn Storage,
            updates: Vec<(String, DerivativeDataConfigUpdate)>,
        ) -> StdResult<()> {
            for (key, update) in updates {
                let mut data = Self::DERIVATIVES.load(storage, &key)?;
                if let Some(underlying_key) = update.underlying_key {
                    self.config
                        .require_valid_router_symbol(querier, &underlying_key)?;
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
                data.rate.require_valid_change(now, rate)?;
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
                supported_pairs.push(StrideStakingDerivativesOracle::get_derivative_data_resp(
                    &key, storage,
                )?);
            }
            Ok(supported_pairs)
        }
    }
}
