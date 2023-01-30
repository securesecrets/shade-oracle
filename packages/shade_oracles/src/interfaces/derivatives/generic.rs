//! Staking derivative oracle that calculate the price of 1 staking derivative token
//! based off its exchange rate with its underlying token.

use super::*;

pub mod msg {
    use crate::{
        asset::{Asset, RawAsset},
        interfaces::common::{config::CommonConfigResponse, PriceResponse, PricesResponse},
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

    pub type DerivativesResponse = Vec<DerivativeData>;

    #[cw_serde]
    pub struct RawDerivativeData {
        pub key: String,
        /// The quote symbol in this asset should be for its underlying.
        /// Ex: stkd-SCRT asset should have the quote symbol for SCRT.
        pub staking_derivative: RawAsset,
    }

    #[cw_serde]
    pub struct DerivativeData {
        pub key: String,
        pub staking_derivative: Asset,
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
        interfaces::common::config::CommonConfig,
    };
    use cosmwasm_std::{Api, QuerierWrapper, StdResult, Storage};
    use ssp::{Item, ItemStorage, Map};

    #[cw_serde]
    pub struct StoredDerivativeData {
        pub derivative: Addr,
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
            Self::set_derivative_data(storage, data.key, contract)
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

        pub fn set_derivative_data(
            storage: &mut dyn Storage,
            key: String,
            derivative: Asset,
        ) -> StdResult<StoredDerivativeData> {
            Self::ASSETS.may_set(storage, &derivative)?;
            let data = StoredDerivativeData {
                derivative: derivative.contract.address,
            };
            Self::DERIVATIVES.save(storage, &key, &data)?;
            CommonConfig::add_supported_key(storage, &key)?;
            Ok(data)
        }

        pub fn get_derivative_data_resp(
            key: &String,
            storage: &dyn Storage,
        ) -> StdResult<DerivativeData> {
            let data = Self::DERIVATIVES.load(storage, key)?;
            let staking_derivative = Self::ASSETS.0.load(storage, &data.derivative)?;
            Ok(DerivativeData {
                key: key.to_string(),
                staking_derivative,
            })
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
