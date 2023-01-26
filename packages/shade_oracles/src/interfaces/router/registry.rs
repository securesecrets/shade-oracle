use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal256, Uint256};
use shade_protocol::{utils::asset::RawContract, Contract};

#[cw_serde]
pub struct Config {
    pub this: Contract,
    pub admin_auth: Contract,
    pub band: Contract,
    pub quote_symbol: String,
}

#[cw_serde]
pub enum RegistryOperation {
    RemoveKeys {
        keys: Vec<String>,
    },
    SetKeys {
        oracle: RawContract,
        keys: Vec<String>,
    },
    SetProtection {
        infos: Vec<ProtectedKeyInfo>,
    },
    RemoveProtection {
        keys: Vec<String>,
    },
}

#[cw_serde]
pub struct UpdateConfig {
    pub admin_auth: Option<RawContract>,
    pub band: Option<RawContract>,
    pub quote_symbol: Option<String>,
}

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
#[cfg(feature = "router")]
mod state {
    use std::{collections::HashMap, vec};

    use crate::{
        impl_global_status,
        interfaces::router::{error::OracleRouterError, msg::KeysResponse},
        interfaces::{common::OraclePrice, router::msg::ProtectedKeysResponse},
    };

    use super::*;
    use cosmwasm_std::{to_binary, Api, Binary, Deps, StdError, StdResult, Storage};
    use secret_storage_plus::{GenericMapStorage, Item, ItemStorage, Map};

    impl_global_status!(OracleRouter, OracleRouterError);

    impl ItemStorage for Config {
        const ITEM: Item<'static, Self> = Item::new("oraclerouterconfig");
    }

    impl<'a> GenericMapStorage<'a, &'a str, Contract> for Oracle {
        const MAP: Map<'static, &'a str, Contract> = Map::new("oraclerouteroracleregistry");
    }

    /// List of explicity supported keys (keys registered to an oracle).
    pub const KEYS: Item<Vec<String>> = Item::new("oraclerouterkeys");

    impl<'a> OracleRouter {
        pub const PROTECTED_KEYS_LIST: Item<'static, Vec<String>> =
            Item::new("oraclerouterprotectedkeyslist");
        pub const PROTECTED_KEYS: Map<'static, &'a str, ProtectedKeyInfo> =
            Map::new("oraclerouterprotectedkeys");
    }

    impl OracleRouter {
        pub fn init_storage(storage: &mut dyn Storage) -> StdResult<()> {
            OracleRouter::PROTECTED_KEYS_LIST.save(storage, &vec![])?;
            KEYS.save(storage, &vec![])?;
            Ok(())
        }

        pub fn update_protected_key(
            storage: &mut dyn Storage,
            key: &str,
            price: Uint256,
        ) -> StdResult<()> {
            Self::PROTECTED_KEYS.update(storage, key, |info| match info {
                None => Err(StdError::generic_err(format!(
                    "Can't update price for unprotected key {}.",
                    key
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
                    let price = Uint256::from_uint128(price.data.rate);
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
            let keys = KEYS.load(deps.storage)?;
            to_binary(&keys)
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
                    let mut current_keys = KEYS.load(storage)?;
                    for key in &keys {
                        Oracle::MAP.remove(storage, key);
                    }
                    current_keys.retain(|k| !keys.contains(k));
                    KEYS.save(storage, &current_keys)?;
                }
                RegistryOperation::SetKeys { oracle, keys } => {
                    let oracle = oracle.into_valid(api)?;
                    let mut current_keys = KEYS.load(storage)?;
                    for key in keys {
                        Oracle::MAP.save(storage, &key, &oracle)?;
                        if !current_keys.contains(&key) {
                            current_keys.push(key);
                        }
                    }
                    KEYS.save(storage, &current_keys)?;
                }
                RegistryOperation::SetProtection { infos } => {
                    let mut protected_keys = Self::PROTECTED_KEYS_LIST.load(storage)?;
                    for info in infos {
                        Self::PROTECTED_KEYS.save(storage, &info.key, &info)?;
                        if !protected_keys.contains(&info.key) {
                            protected_keys.push(info.key);
                        }
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
                new_config.admin_auth = admin_auth.into_valid(api)?;
            }
            if let Some(band) = config.band {
                new_config.band = band.into_valid(api)?;
            }
            new_config.quote_symbol = config.quote_symbol.unwrap_or(new_config.quote_symbol);
            self.config = new_config;
            Ok(self)
        }
    }
}
