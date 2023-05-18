use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal256, Uint256};
use shade_toolkit::Contract;

#[cw_serde]
pub struct OracleRouter {
    pub config: Config,
}

#[cw_serde]
pub struct ProtectedKeyInfo {
    pub key: String,
    pub deviation: Decimal256,
    pub price: Uint256,
}

impl ProtectedKeyInfo {
    pub fn new(key: String, deviation: Decimal256, price: Uint256) -> Self {
        Self {
            key,
            deviation,
            price,
        }
    }
}

pub struct Oracle;

#[cfg(feature = "router")]
pub use state::*;

use super::msg::Config;
#[cfg(feature = "router")]
mod state {
    use std::{
        collections::{HashMap, HashSet},
        vec,
    };

    use crate::{
        impl_global_status,
        interfaces::{common::OraclePrice, router::msg::RegistryOperation},
        interfaces::{
            common::{PriceResponse, PricesResponse},
            router::{error::OracleRouterError, msg::UpdateConfig},
        },
    };

    use super::*;
    use cosmwasm_std::{
        to_binary, Api, Binary, Deps, QuerierWrapper, StdError, StdResult, Storage,
    };
    use secret_borsh_storage::BorshItem;
    use secret_storage_plus::{GenericMapStorage, Item, ItemStorage, Map};

    impl_global_status!(OracleRouter, OracleRouterError);

    impl ItemStorage for Config {
        const ITEM: Item<'static, Self> = Item::new("oraclerouterconfig");
    }

    impl<'a> GenericMapStorage<'a, &'a str, Contract> for Oracle {
        const MAP: Map<'static, &'a str, Contract> = Map::new("oraclerouteroracleregistry");
    }

    impl<'a> OracleRouter {
        /// List of explicity supported keys (keys registered to an oracle).
        pub const KEYS: BorshItem<'static, HashSet<String>> = BorshItem::new("oraclerouterkeys");
        pub const PROTECTED_KEYS_LIST: BorshItem<'static, HashSet<String>> =
            BorshItem::new("oraclerouterprotectedkeyslist");
        pub const PROTECTED_KEYS: Map<'static, &'a str, ProtectedKeyInfo> =
            Map::new("oraclerouterprotectedkeys");
    }

    impl OracleRouter {
        pub fn init_storage(storage: &mut dyn Storage) -> StdResult<()> {
            Self::PROTECTED_KEYS_LIST.save(storage, &HashSet::new())?;
            Self::KEYS.save(storage, &HashSet::new())?;
            Ok(())
        }

        pub fn update_protected_key(
            storage: &mut dyn Storage,
            key: &str,
            price: Uint256,
        ) -> StdResult<()> {
            Self::PROTECTED_KEYS.update(storage, key, |info| match info {
                None => Err(StdError::generic_err(format!(
                    "Can't update price for unprotected key {key}.",
                ))),
                Some(mut info) => {
                    info.price = price;
                    Ok(info)
                }
            })?;
            Ok(())
        }
        pub fn try_deviation_test(storage: &dyn Storage, price: &OraclePrice) -> StdResult<()> {
            let info = Self::PROTECTED_KEYS.may_load(storage, &price.key)?;
            match info {
                None => Ok(()),
                Some(info) => {
                    let price = price.data.rate;
                    let abs_diff = if info.price > price {
                        info.price - price
                    } else {
                        price - info.price
                    };
                    if abs_diff.is_zero() {
                        Ok(())
                    } else {
                        let deviation = Decimal256::from_ratio(abs_diff, info.price);
                        if deviation > info.deviation {
                            Err(StdError::generic_err(
                                OracleRouterError::ProtectedPriceDeviation {
                                    actual: price,
                                    expected: info.price,
                                    deviation: info.deviation,
                                    actual_deviation: deviation,
                                }
                                .to_string(),
                            ))
                        } else {
                            Ok(())
                        }
                    }
                }
            }
        }

        pub fn load(storage: &dyn Storage) -> StdResult<Self> {
            let config = Config::load(storage)?;
            Ok(OracleRouter { config })
        }

        pub fn get_keys(deps: Deps) -> StdResult<Binary> {
            let keys = Self::KEYS.load(deps.storage)?;
            to_binary(&keys.into_iter().collect::<Vec<String>>())
        }

        pub fn get_protected_keys(deps: Deps) -> StdResult<Binary> {
            let keys = Self::PROTECTED_KEYS_LIST.load(deps.storage)?;
            let mut protected_keys = vec![];
            for key in keys {
                if let Some(info) = Self::PROTECTED_KEYS.may_load(deps.storage, &key)? {
                    protected_keys.push(info);
                }
            }
            to_binary(&protected_keys)
        }

        pub fn resolve_registry_operation(
            api: &dyn Api,
            storage: &mut dyn Storage,
            operation: RegistryOperation,
        ) -> StdResult<()> {
            match operation {
                RegistryOperation::RemoveKeys { keys } => {
                    let mut current_keys = Self::KEYS.load(storage)?;
                    for key in &keys {
                        Oracle::MAP.remove(storage, key);
                    }
                    current_keys.retain(|k| !keys.contains(k));
                    Self::KEYS.save(storage, &current_keys)?;
                }
                RegistryOperation::SetKeys { oracle, keys } => {
                    let oracle = oracle.validate(api)?;
                    let mut current_keys = Self::KEYS.load(storage)?;
                    for key in keys {
                        Oracle::MAP.save(storage, &key, &oracle)?;
                        current_keys.insert(key);
                    }
                    Self::KEYS.save(storage, &current_keys)?;
                }
                RegistryOperation::SetProtection { infos } => {
                    let mut protected_keys = Self::PROTECTED_KEYS_LIST.load(storage)?;
                    for info in infos {
                        Self::PROTECTED_KEYS.save(storage, &info.key, &info)?;
                        protected_keys.insert(info.key);
                    }
                    Self::PROTECTED_KEYS_LIST.save(storage, &protected_keys)?;
                }
                RegistryOperation::RemoveProtection { keys } => {
                    let mut protected_keys = Self::PROTECTED_KEYS_LIST.load(storage)?;
                    let mut keys_to_remove = vec![];
                    for key in keys {
                        Self::PROTECTED_KEYS.remove(storage, &key);
                        keys_to_remove.push(key);
                    }
                    protected_keys.retain(|k| !keys_to_remove.contains(k));
                    Self::PROTECTED_KEYS_LIST.save(storage, &protected_keys)?;
                }
            }
            Ok(())
        }

        pub fn get_oracle(&self, storage: &dyn Storage, key: &str) -> StdResult<Contract> {
            match Oracle::may_load(storage, key)? {
                Some(contract) => Ok(contract),
                None => Ok(self.config.this.clone()),
            }
        }

        pub fn group_keys_by_oracle(
            &self,
            storage: &dyn Storage,
            keys: &[String],
        ) -> StdResult<HashMap<Contract, Vec<String>>> {
            // Maps oracle to the symbols it is responsible for
            let mut map: HashMap<Contract, Vec<String>> = HashMap::new();

            for current_key in keys {
                let oracle = self.get_oracle(storage, current_key)?;

                // Get the current vector of symbols at that oracle and add the current key to it
                map.entry(oracle)
                    .or_insert(vec![])
                    .push(current_key.to_string());
            }

            Ok(map)
        }

        pub fn update_config(mut self, api: &dyn Api, config: UpdateConfig) -> StdResult<Self> {
            let mut new_config = self.config;
            if let Some(admin_auth) = config.admin_auth {
                new_config.admin_auth = admin_auth.validate(api)?;
            }
            if let Some(provider) = config.provider {
                new_config.provider = provider.validate(api)?;
            }
            new_config.quote_symbol = config.quote_symbol.unwrap_or(new_config.quote_symbol);
            self.config = new_config;
            Ok(self)
        }

        pub fn query_provider_price(
            &self,
            querier: &QuerierWrapper,
            key: String,
        ) -> StdResult<PriceResponse> {
            let Config {
                quote_symbol,
                provider,
                ..
            } = &self.config;
            let symbol_pair = (key.as_str(), quote_symbol.as_str());
            let resp = provider.reference_data(querier, symbol_pair)?;
            Ok(OraclePrice::new(key, resp))
        }

        pub fn query_provider_prices(
            &self,
            querier: &QuerierWrapper,
            keys: Vec<String>,
        ) -> StdResult<PricesResponse> {
            let Config {
                quote_symbol,
                provider,
                ..
            } = &self.config;
            let mut prices: Vec<OraclePrice> = vec![];
            let symbol_pairs: Vec<(String, String)> = keys
                .iter()
                .map(|key| (key.to_string(), quote_symbol.to_string()))
                .collect();

            let data = provider.reference_data_bulk(querier, symbol_pairs)?;

            for (index, key) in keys.iter().enumerate() {
                prices.push(OraclePrice::new(key.to_string(), data[index].clone()));
            }
            Ok(prices)
        }
    }
}
