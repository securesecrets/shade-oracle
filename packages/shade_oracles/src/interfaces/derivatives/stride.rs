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
        APY(Vec<(String, Decimal256)>),
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
        pub rate_update_frequency: u64,
        pub rate_timeout: u64,
        pub apy: Decimal256,
        pub apy_update_frequency: u64,
        pub apy_max_change: Decimal256,
    }

    #[cw_serde]
    pub struct DerivativeDataConfigUpdate {
        pub underlying_key: Option<String>,
        pub rate_update_frequency: Option<u64>,
        pub rate_timeout: Option<u64>,
        pub apy_update_frequency: Option<u64>,
        pub apy_max_change: Option<Decimal256>,
    }

    #[cw_serde]
    pub struct DerivativeData {
        pub key: String,
        pub underlying_key: String,
        pub rate: DerivativeRate,
        pub apy: DerivativeApy,
    }

    #[cw_serde]
    pub struct DerivativeRate {
        pub value: Decimal256,
        pub update_frequency: u64,
        pub timeout: u64,
        pub last_updated: u64,
    }

    #[cw_serde]
    pub struct DerivativeApy {
        pub value: Decimal256,
        pub update_frequency: u64,
        pub max_change: Decimal256,
        pub last_updated: u64,
    }

    #[cw_serde]
    pub enum BotPermission {
        UpdateRates,
        UpdateAPY,
    }

    impl ToString for BotPermission {
        fn to_string(&self) -> String {
            match self {
                BotPermission::UpdateRates => "SHADE_ORACLES_STRIDE_RATES_BOT".to_string(),
                BotPermission::UpdateAPY => "SHADE_ORACLES_STRIDE_APY_BOT".to_string(),
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

    /// 5.01% is the maximum downside price change allowed for a derivative.
    pub const MAX_DOWNSIDE: Decimal256 = Decimal256::new(Uint256::from_u128(51 * 10u128.pow(15)));

    impl DerivativeApy {
        pub fn new(
            value: Decimal256,
            update_frequency: u64,
            max_change: Decimal256,
            last_updated: u64,
        ) -> StdResult<Self> {
            let mut apy = Self {
                value,
                update_frequency,
                max_change,
                last_updated,
            };
            apy.set_value(value)?;
            apy.set_max_change(max_change)?;
            Ok(apy)
        }
        pub fn set_value(&mut self, value: Decimal256) -> StdResult<()> {
            if value.is_zero() {
                return Err(StdError::generic_err("APY must be greater than 0"));
            };
            self.value = value;
            Ok(())
        }
        pub fn set_max_change(&mut self, max_change: Decimal256) -> StdResult<()> {
            if max_change.is_zero() {
                return Err(StdError::generic_err("Max change must be greater than 0"));
            };
            self.max_change = max_change;
            Ok(())
        }
        pub fn require_valid_change(&self, now: u64, new_apy: Decimal256) -> StdResult<()> {
            let time_since_updated = now - self.last_updated;
            if time_since_updated < self.update_frequency {
                return Err(StdError::generic_err(format!(
                    "Derivative APY is being updated too frequently. Please wait {0} seconds before updating again.",
                    self.update_frequency - time_since_updated
                )));
            }
            if self.value.eq(&new_apy) {
                Ok(())
            } else if self.value > new_apy {
                let change = self.value - new_apy;
                if change > self.max_change {
                    Err(StdError::generic_err(format!(
                        "Derivative APY is changing too much. Maximum downside is {0}. Attempted change is {change}.", self.max_change
                    )))
                } else {
                    Ok(())
                }
            } else {
                let change = new_apy - self.value;
                if change > self.max_change {
                    Err(StdError::generic_err(format!(
                        "Derivative rate is changing too much. Maximum upside is {0}. Attempted change is {change}.", self.max_change
                    )))
                } else {
                    Ok(())
                }
            }
        }
    }

    impl DerivativeRate {
        pub fn new(
            value: Decimal256,
            update_frequency: u64,
            timeout: u64,
            last_updated: u64,
        ) -> StdResult<Self> {
            let mut rate = Self {
                value,
                update_frequency,
                timeout,
                last_updated,
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
            apy: Decimal256,
            new_rate: Decimal256,
        ) -> StdResult<()> {
            let time_since_updated = now - self.last_updated;
            if time_since_updated < self.update_frequency {
                return Err(StdError::generic_err(format!(
                    "Derivative rate is being updated too frequently. Please wait {0} seconds before updating again.",
                    self.update_frequency - time_since_updated
                )));
            }
            let hours_since_updated = Decimal256::from_ratio(time_since_updated, 3600u128);
            let y = Decimal256::from_ratio(
                hours_since_updated.atomics(),
                Uint256::from_u128(24 * 365 * 10u128.pow(18)),
            );
            // Multiplying by 2 for buffer.
            let max_upside = Decimal256::new(Uint256::from_u128(2u128)) * apy * y;
            if self.value.eq(&new_rate) {
                Ok(())
            } else if self.value > new_rate {
                let change = self.value - new_rate;
                if change > MAX_DOWNSIDE {
                    Err(StdError::generic_err(format!(
                        "Derivative rate is changing too much. Maximum downside is {MAX_DOWNSIDE}. Attempted change is {change}."
                    )))
                } else {
                    Ok(())
                }
            } else {
                let change = new_rate - self.value;
                if change > max_upside {
                    Err(StdError::generic_err(format!(
                        "Derivative rate is changing too much. Maximum upside is {max_upside}. Attempted change is {change}."
                    )))
                } else {
                    Ok(())
                }
            }
        }
    }

    impl DerivativeData {
        pub fn new(
            key: String,
            underlying_key: String,
            initial_rate: Decimal256,
            rate_update_frequency: u64,
            rate_timeout: u64,
            apy: Decimal256,
            apy_update_frequency: u64,
            apy_max_change: Decimal256,
            last_updated: u64,
        ) -> StdResult<Self> {
            let rate = DerivativeRate::new(
                initial_rate,
                rate_update_frequency,
                rate_timeout,
                last_updated,
            )?;
            let apy = DerivativeApy::new(apy, apy_update_frequency, apy_max_change, last_updated)?;
            Ok(Self {
                key,
                underlying_key,
                rate,
                apy,
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
                    data.rate_update_frequency,
                    data.rate_timeout,
                    data.apy,
                    data.apy_update_frequency,
                    data.apy_max_change,
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
                if let Some(pos) = supported_keys.iter().position(|k| key.eq(k)) {
                    Self::DERIVATIVES.remove(storage, &key);
                    supported_keys.swap_remove(pos);
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
                if let Some(rate_update_frequency) = update.rate_update_frequency {
                    data.rate.update_frequency = rate_update_frequency;
                }
                if let Some(rate_timeout) = update.rate_timeout {
                    data.rate.timeout = rate_timeout;
                }
                if let Some(apy_update_frequency) = update.apy_update_frequency {
                    data.apy.update_frequency = apy_update_frequency;
                }
                if let Some(apy_max_change) = update.apy_max_change {
                    data.apy.set_max_change(apy_max_change)?;
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
                data.rate.require_valid_change(now, data.apy.value, rate)?;
                data.rate.last_updated = now;
                data.rate.value = rate;
                Self::DERIVATIVES.save(storage, &key, &data)?;
            }
            Ok(())
        }

        pub fn update_apys(
            storage: &mut dyn Storage,
            now: u64,
            apys: Vec<(String, Decimal256)>,
        ) -> StdResult<()> {
            for (key, apy) in apys {
                let mut data = Self::DERIVATIVES.load(storage, &key)?;
                data.apy.require_valid_change(now, apy)?;
                data.apy.last_updated = now;
                data.apy.value = apy;
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
