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
        Timeouts(Vec<(String, u64)>),
        Frequencies(Vec<(String, u64)>),
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
        pub apy: Decimal256,
        pub update_frequency: u64,
        pub timeout: u64,
    }

    #[cw_serde]
    pub struct DerivativeData {
        pub key: String,
        pub underlying_key: String,
        pub rate: Decimal256,
        pub apy: Decimal256,
        pub update_frequency: u64,
        pub timeout: u64,
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

    impl DerivativeData {
        pub fn new(
            key: String,
            underlying_key: String,
            rate: Decimal256,
            apy: Decimal256,
            update_frequency: u64,
            timeout: u64,
            last_updated: u64,
        ) -> StdResult<Self> {
            if rate.is_zero() {
                return Err(StdError::generic_err("Rate must be greater than 0"));
            };
            if apy.is_zero() {
                return Err(StdError::generic_err("APY must be greater than 0"));
            };
            Ok(Self {
                key,
                underlying_key,
                rate,
                apy,
                update_frequency,
                timeout,
                last_updated,
            })
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
        pub fn require_valid_price_change(&self, new_rate: Decimal256, now: u64) -> StdResult<()> {
            let time_since_updated = now - self.last_updated;
            if time_since_updated < self.update_frequency {
                return Err(StdError::generic_err(format!(
                    "Derivative rate is being updated too frequently. Please wait {0} seconds before updating again.",
                    self.update_frequency - time_since_updated
                )));
            }
            let hours_since_updated = time_since_updated / 3600;
            let y = Decimal256::from_ratio(hours_since_updated, Uint256::from_u128(24 * 365));
            let max_upside = self.apy / y;
            if self.rate > new_rate {
                let change = self.rate - new_rate;
                if change > MAX_DOWNSIDE {
                    Err(StdError::generic_err(format!(
                        "Derivative rate is changing too much. Maximum downside is {MAX_DOWNSIDE}. Attempted change is {change}."
                    )))
                } else {
                    Ok(())
                }
            } else {
                let change = new_rate - self.rate;
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

        pub fn set_derivative_data(
            storage: &mut dyn Storage,
            data: RawDerivativeData,
            now: u64,
        ) -> StdResult<DerivativeData> {
            let data = DerivativeData::new(
                data.key,
                data.underlying_key,
                data.initial_rate,
                data.apy,
                data.update_frequency,
                data.timeout,
                now,
            )?;
            Self::DERIVATIVES.save(storage, &data.key, &data)?;
            CommonConfig::add_supported_key(storage, &data.key)?;
            Ok(data)
        }

        pub fn get_derivative_data_resp(
            key: &String,
            storage: &dyn Storage,
        ) -> StdResult<DerivativeData> {
            let data = Self::DERIVATIVES.load(storage, key)?;
            Ok(data)
        }

        pub fn update_rates(
            storage: &mut dyn Storage,
            now: u64,
            rates: Vec<(String, Decimal256)>,
        ) -> StdResult<()> {
            for (key, rate) in rates {
                let mut data = Self::DERIVATIVES.load(storage, &key)?;
                data.require_valid_price_change(rate, now)?;
                data.last_updated = now;
                data.rate = rate;
                Self::DERIVATIVES.save(storage, &key, &data)?;
            }
            Ok(())
        }

        pub fn update_frequencies(
            storage: &mut dyn Storage,
            frequencies: Vec<(String, u64)>,
        ) -> StdResult<()> {
            for (key, frequency) in frequencies {
                let mut data = Self::DERIVATIVES.load(storage, &key)?;
                data.update_frequency = frequency;
                Self::DERIVATIVES.save(storage, &key, &data)?;
            }
            Ok(())
        }

        pub fn update_apys(
            storage: &mut dyn Storage,
            apys: Vec<(String, Decimal256)>,
        ) -> StdResult<()> {
            for (key, apy) in apys {
                let mut data = Self::DERIVATIVES.load(storage, &key)?;
                data.apy = apy;
                Self::DERIVATIVES.save(storage, &key, &data)?;
            }
            Ok(())
        }

        pub fn update_timeouts(
            storage: &mut dyn Storage,
            timeouts: Vec<(String, u64)>,
        ) -> StdResult<()> {
            for (key, timeout) in timeouts {
                let mut data = Self::DERIVATIVES.load(storage, &key)?;
                data.timeout = timeout;
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
