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

    #[cfg(test)]
    #[cfg(feature = "derivatives")]
    mod test {
        use super::*;
        use cosmwasm_std::testing::mock_dependencies;

        #[test]
        fn test_get_derivative_data_resp_none_rate_timeout() {
            let mut deps = mock_dependencies();
            let mut_deps = deps.as_mut();
            let asset = Asset {
                contract: Contract {
                    address: Addr::unchecked("addr"),
                    code_hash: "code_hash".to_string(),
                },
                quote_symbol: "quote_symbol".to_string(),
                decimals: 10u8,
            };
            let key = "key".to_string();
            let _data = StakingDerivativesOracle::set_derivative_data(
                mut_deps.storage,
                key.clone(),
                asset.clone(),
                None,
                None,
            )
            .unwrap();
            let resp =
                StakingDerivativesOracle::get_derivative_data_resp(&key, mut_deps.storage).unwrap();
            assert_eq!(resp.key, key);
            assert_eq!(resp.staking_derivative, asset);
            assert_eq!(resp.timeout, None);
            assert_eq!(resp.rate, None);
        }

        #[test]
        fn test_set_derivative_data() {
            let mut deps = mock_dependencies();
            let mut_deps = deps.as_mut();
            let asset = Asset {
                contract: Contract {
                    address: Addr::unchecked("addr"),
                    code_hash: "code_hash".to_string(),
                },
                quote_symbol: "quote_symbol".to_string(),
                decimals: 10u8,
            };
            let key = "key".to_string();
            let data = StakingDerivativesOracle::set_derivative_data(
                mut_deps.storage,
                key.clone(),
                asset.clone(),
                Some(OraclePrice::default()),
                Some(0),
            )
            .unwrap();
            assert_eq!(data.derivative, asset.contract.address);
            assert_eq!(data.rate, OraclePrice::default());
            assert_eq!(data.timeout, 0);
            let stored_asset = StakingDerivativesOracle::ASSETS
                .0
                .load(mut_deps.storage, &data.derivative)
                .unwrap();
            assert_eq!(stored_asset, asset);
            let stored_data = StakingDerivativesOracle::DERIVATIVES
                .load(mut_deps.storage, &key)
                .unwrap();
            assert_eq!(stored_data, data);
            let supported_keys = CommonConfig::SUPPORTED_KEYS.load(mut_deps.storage).unwrap();
            assert_eq!(supported_keys, vec![key]);
        }

        #[test]
        fn test_remove_keys() {
            let mut deps = mock_dependencies();
            let mut_deps = deps.as_mut();
            let mut supported_keys = vec![];
            for i in 0..10 {
                let key = format!("key_{}", i);
                supported_keys.push(key.clone());
                let data = StoredDerivativeData {
                    derivative: Addr::unchecked(format!("addr_{}", i)),
                    rate: OraclePrice::default(),
                    timeout: 0,
                };
                StakingDerivativesOracle::DERIVATIVES
                    .save(mut_deps.storage, &key, &data)
                    .unwrap();
            }
            CommonConfig::SUPPORTED_KEYS
                .save(mut_deps.storage, &supported_keys)
                .unwrap();
            let keys = vec!["key_1".to_string(), "key_3".to_string()];
            StakingDerivativesOracle::remove_keys(mut_deps.storage, keys).unwrap();
            let supported_keys = CommonConfig::SUPPORTED_KEYS.load(mut_deps.storage).unwrap();
            // key_1 and key_3 should be removed, not preserving order
            assert_eq!(supported_keys.len(), 8);
            assert!(supported_keys.contains(&"key_0".to_string()));
            assert!(supported_keys.contains(&"key_2".to_string()));
            assert!(supported_keys.contains(&"key_4".to_string()));
            assert!(supported_keys.contains(&"key_5".to_string()));
            assert!(supported_keys.contains(&"key_6".to_string()));
            assert!(supported_keys.contains(&"key_7".to_string()));
            assert!(supported_keys.contains(&"key_8".to_string()));
            assert!(supported_keys.contains(&"key_9".to_string()));
            assert!(!supported_keys.contains(&"key_1".to_string()));
            assert!(!supported_keys.contains(&"key_3".to_string()));
        }

        #[test]
        fn test_update_rates() {
            let mut deps = mock_dependencies();
            let mut_deps = deps.as_mut();
            let mut supported_keys = vec![];
            for i in 0..10 {
                let key = format!("key_{}", i);
                supported_keys.push(key.clone());
                let data = StoredDerivativeData {
                    derivative: Addr::unchecked(format!("addr_{}", i)),
                    rate: OraclePrice::default(),
                    timeout: 0,
                };
                StakingDerivativesOracle::DERIVATIVES
                    .save(mut_deps.storage, &key, &data)
                    .unwrap();
            }
            CommonConfig::SUPPORTED_KEYS
                .save(mut_deps.storage, &supported_keys)
                .unwrap();
            let rates = vec![
                DerivativeExchangeRate {
                    symbol: "key_1".to_string(),
                    price: Uint128::from(1u128),
                },
                DerivativeExchangeRate {
                    symbol: "key_2".to_string(),
                    price: Uint128::from(2u128),
                },
                DerivativeExchangeRate {
                    symbol: "key_3".to_string(),
                    price: Uint128::from(3u128),
                },
            ];
            let now = 100;
            StakingDerivativesOracle::update_rates(mut_deps.storage, now, rates).unwrap();
            for i in 0..10 {
                let key = format!("key_{}", i);
                let data = StakingDerivativesOracle::DERIVATIVES
                    .load(mut_deps.storage, &key)
                    .unwrap();
                if i == 1 {
                    assert_eq!(data.rate.data.rate, Uint128::from(1u128));
                    assert_eq!(data.rate.data.last_updated_base, now);
                    assert_eq!(data.rate.data.last_updated_quote, now);
                } else if i == 2 {
                    assert_eq!(data.rate.data.rate, Uint128::from(2u128));
                    assert_eq!(data.rate.data.last_updated_base, now);
                    assert_eq!(data.rate.data.last_updated_quote, now);
                } else if i == 3 {
                    assert_eq!(data.rate.data.rate, Uint128::from(3u128));
                    assert_eq!(data.rate.data.last_updated_base, now);
                    assert_eq!(data.rate.data.last_updated_quote, now);
                } else {
                    assert_eq!(data.rate, OraclePrice::default());
                }
            }
        }

        #[test]
        fn test_get_supported_derivatives() {
            let mut deps = mock_dependencies();
            let mut_deps = deps.as_mut();
            // Populate storage with 10 derivatives using the set_derivative_data function
            // and check that the get_supported_derivatives function returns the same
            for i in 0..10 {
                let key = format!("key_{}", i);
                let asset = Asset::new(Contract::new(&Addr::unchecked(key.clone()), &"symbol".to_string()), 10, key.clone());
                StakingDerivativesOracle::set_derivative_data(
                    mut_deps.storage,
                    key.clone(),
                    asset,
                    Some(OraclePrice::default()),
                    None
                ).unwrap();
            }
            let resp =
                StakingDerivativesOracle::get_supported_derivatives(mut_deps.storage).unwrap();
            assert_eq!(resp.len(), 10);
        }
    }
}
