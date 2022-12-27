//! Staking derivative oracles calculate the price of 1 staking derivative token
//! based off its exchange rate with its underlying token.

use super::*;

pub mod msg {
    use crate::{
        asset::{Asset, RawAsset},
        interfaces::common::{
            config::CommonConfigResponse, OraclePrice, PriceResponse, PricesResponse,
        },
    };

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
        UpdateAssets(Vec<RawAsset>),
        /// Optional. Used by bot if we can't get the rate from the contract itself.
        UpdateRates(Vec<DerivativeExchangeRate>),
        UpdateConfig(RawContract),
        SetStatus(bool),
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

    #[cw_serde]
    pub struct DerivativeExchangeRate {
        pub symbol: String,
        pub price: Uint128,
    }

    pub type DerivativesResponse = Vec<DerivativeData>;

    #[cw_serde]
    pub struct RawDerivativeData {
        pub key: String,
        /// The quote symbol for this asset should be for its underlying.
        /// Ex: stkd-SCRT asset should have the quote symbol for SCRT.
        pub staking_derivative: RawAsset,
        pub initial_rate: Option<OraclePrice>,
        pub timeout: Option<u64>,
    }

    #[cw_serde]
    pub struct DerivativeData {
        pub key: String,
        pub staking_derivative: Asset,
        pub rate: Option<OraclePrice>,
        pub timeout: Option<u64>,
    }
}

#[cfg(feature = "derivatives")]
pub use msg::*;
#[cfg(feature = "derivatives")]
pub use state::*;
#[cfg(feature = "derivatives")]
mod state {
    use super::*;
    use crate::{
        asset::{Asset, Assets, RawAsset},
        interfaces::{
            band::ReferenceData,
            common::{config::CommonConfig, OraclePrice},
        },
    };
    use cosmwasm_std::{Api, QuerierWrapper, StdResult, Storage};
    use ssp::{Item, ItemStorage, Map};

    #[cw_serde]
    pub struct StoredDerivativeData {
        pub derivative: Addr,
        /// Unused if rate is available directly from derivatives contract.
        pub rate: OraclePrice,
        /// Unused if rate is available directly from derivatives contract.
        pub timeout: u64,
    }

    #[cw_serde]
    pub struct StakingDerivativesOracle {
        pub config: CommonConfig,
    }

    impl ItemStorage for StakingDerivativesOracle {
        const ITEM: Item<'static, Self> = Item::new("staking_derivatives_oracle");
    }

    impl<'a> StakingDerivativesOracle {
        pub const ASSETS: Assets<'static, 'a> = Assets::new("pair_assets");
        // Keyed by its symbol.
        pub const DERIVATIVES: Map<'static, &'a str, StoredDerivativeData> =
            Map::new("derivatives");
    }

    impl StakingDerivativesOracle {
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

        pub fn update_asset_symbol(
            &self,
            storage: &mut dyn Storage,
            api: &dyn Api,
            querier: &QuerierWrapper,
            asset: RawAsset,
        ) -> StdResult<()> {
            let asset = asset.into_asset(&self.config.router, querier, api)?;
            Self::ASSETS.update_existing_asset(
                storage,
                querier,
                &self.config.router,
                &asset.contract.address,
                &asset.quote_symbol,
            )?;
            Ok(())
        }

        pub fn set_derivative_data(
            storage: &mut dyn Storage,
            key: String,
            derivative: Asset,
            rate: Option<OraclePrice>,
            timeout: Option<u64>,
        ) -> StdResult<StoredDerivativeData> {
            Self::ASSETS.may_set(storage, &derivative)?;
            let data = StoredDerivativeData {
                derivative: derivative.contract.address,
                rate: rate.unwrap_or_default(),
                timeout: timeout.unwrap_or_default(),
            };
            Self::DERIVATIVES.save(storage, &key, &data)?;
            CommonConfig::add_supported_key(storage, &key)?;
            Ok(data)
        }

        /// Performs validation and saves the data to storage.
        pub fn validate_and_set_derivative_data(
            &self,
            storage: &mut dyn Storage,
            api: &dyn Api,
            querier: &QuerierWrapper,
            data: RawDerivativeData,
        ) -> StdResult<StoredDerivativeData> {
            let contract = data
                .staking_derivative
                .into_asset(&self.config.router, querier, api)?;
            Self::set_derivative_data(storage, data.key, contract, data.initial_rate, data.timeout)
        }

        pub fn get_derivative_data_resp(
            key: &String,
            storage: &dyn Storage,
        ) -> StdResult<DerivativeData> {
            let data = Self::DERIVATIVES.load(storage, key)?;
            let staking_derivative = Self::ASSETS.0.load(storage, &data.derivative)?;
            let timeout = if data.timeout == u64::default() {
                None
            } else {
                Some(data.timeout)
            };
            let rate = if data.rate == OraclePrice::default() {
                None
            } else {
                Some(data.rate)
            };
            Ok(DerivativeData {
                key: key.to_string(),
                staking_derivative,
                timeout,
                rate,
            })
        }

        pub fn update_rates(
            storage: &mut dyn Storage,
            now: u64,
            rates: Vec<DerivativeExchangeRate>,
        ) -> StdResult<()> {
            for rate in rates {
                let mut data = Self::DERIVATIVES.load(storage, &rate.symbol)?;
                data.rate = OraclePrice {
                    key: rate.symbol.clone(),
                    data: ReferenceData {
                        rate: rate.price,
                        last_updated_base: now,
                        last_updated_quote: now,
                    },
                };
                Self::DERIVATIVES.save(storage, &rate.symbol, &data)?;
            }
            Ok(())
        }

        pub fn get_supported_derivatives(storage: &dyn Storage) -> StdResult<DerivativesResponse> {
            let keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
            let mut supported_pairs = vec![];
            for key in keys {
                supported_pairs.push(StakingDerivativesOracle::get_derivative_data_resp(
                    &key, storage,
                )?);
            }
            Ok(supported_pairs)
        }
    }
}
