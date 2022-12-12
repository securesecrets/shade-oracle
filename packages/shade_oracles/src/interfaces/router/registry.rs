use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal256, Uint256};
use shade_protocol::Contract;

#[cw_serde]
pub struct Config {
    pub this: Contract,
    pub admin_auth: Contract,
    pub band: Contract,
    pub quote_symbol: String,
}

#[cw_serde]
pub enum RegistryOperation {
    Remove {
        key: String,
    },
    Replace {
        oracle: Contract,
        key: String,
    },
    Add {
        oracle: Contract,
        key: String,
    },
    Protect {
        key: String,
        deviation: Decimal256,
        initial_price: Uint256,
    },
    UpdateProtection {
        key: String,
        deviation: Option<Decimal256>,
        price: Option<Uint256>,
    },
    RemoveProtection {
        key: String,
    },
}

#[cw_serde]
pub struct UpdateConfig {
    pub admin_auth: Option<Contract>,
    pub band: Option<Contract>,
    pub quote_symbol: Option<String>,
}

#[cw_serde]
pub struct OracleRouter {
    pub config: Config,
}

#[cw_serde]
pub struct ProtectedKeyInfo {
    pub deviation: Decimal256,
    pub price: Uint256,
}

pub struct Oracle;

#[cfg(feature = "router")]
pub use state::*;
#[cfg(feature = "router")]
mod state {
    use std::collections::HashMap;

    use crate::{
        common::OraclePrice,
        impl_global_status,
        interfaces::router::{
            error::{OracleRouterError, OracleRouterResult},
            msg::KeysResponse,
        },
    };

    use super::*;
    use cosmwasm_std::{to_binary, Binary, Deps, StdError, StdResult, Storage};
    use schemars::_serde_json::de;
    use secret_storage_plus::{GenericMapStorage, Item, ItemStorage, Map};

    impl_global_status!(OracleRouter, OracleRouterError);

    impl ItemStorage for Config {
        const ITEM: Item<'static, Self> = Item::new("oraclerouterconfig");
    }

    impl<'a> GenericMapStorage<'a, &'a str, Contract> for Oracle {
        const MAP: Map<'static, &'a str, Contract> = Map::new("oraclerouteroracleregistry");
    }

    pub const KEYS: Item<Vec<String>> = Item::new("oraclerouterkeys");

    impl<'a> OracleRouter {
        pub const PROTECTED_KEYS: Map<'static, &'a str, ProtectedKeyInfo> =
            Map::new("oraclerouterprotectedkeys");
    }

    impl OracleRouter {
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
            to_binary(&KeysResponse { keys })
        }

        pub fn resolve_registry_operation(
            storage: &mut dyn Storage,
            operation: RegistryOperation,
        ) -> StdResult<()> {
            match operation {
                RegistryOperation::Remove { key } => {
                    Oracle::MAP.remove(storage, &key);
                    KEYS.update(storage, |mut keys| -> StdResult<_> {
                        keys.retain(|k| key.ne(k));
                        Ok(keys)
                    })?;
                }
                RegistryOperation::Replace { oracle, key } => {
                    Oracle::MAP.update(storage, &key, |old_oracle| -> StdResult<_> {
                        match old_oracle {
                            Some(_) => Ok(oracle),
                            None => Err(StdError::generic_err(format!(
                                "Cannot replace oracle at key {} if there wasn't one already there.",
                                key
                            ))),
                        }
                    })?;
                    KEYS.update(storage, |mut keys| -> StdResult<_> {
                        let position = keys.iter().position(|k| key.eq(k));
                        if let Some(index) = position {
                            keys.swap_remove(index);
                            keys.push(key);
                        }
                        Ok(keys)
                    })?;
                }
                RegistryOperation::Add { oracle, key } => {
                    Oracle::MAP.update(storage, &key, |old_oracle| -> StdResult<_> {
                        match old_oracle {
                            Some(_) => Err(StdError::generic_err(format!(
                                "An oracle already exists at the key - {}.",
                                key
                            ))),
                            None => Ok(oracle),
                        }
                    })?;
                    KEYS.update(storage, |mut keys| -> StdResult<_> {
                        keys.push(key);
                        Ok(keys)
                    })?;
                }
                RegistryOperation::Protect {
                    key,
                    deviation,
                    initial_price,
                } => Self::PROTECTED_KEYS.save(
                    storage,
                    &key,
                    &ProtectedKeyInfo {
                        deviation,
                        price: initial_price,
                    },
                )?,
                RegistryOperation::UpdateProtection {
                    key,
                    deviation,
                    price,
                } => {
                    Self::PROTECTED_KEYS.update(storage, &key, |info| match info {
                        None => Err(StdError::generic_err(format!(
                            "Cannot update protection for key {} that hasn't been protected yet.",
                            key
                        ))),
                        Some(mut info) => {
                            info.deviation = deviation.unwrap_or(info.deviation);
                            info.price = price.unwrap_or(info.price);
                            Ok(info)
                        }
                    })?;
                }
                RegistryOperation::RemoveProtection { key } => {
                    Self::PROTECTED_KEYS.remove(storage, &key)
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

        pub fn update_config(mut self, config: UpdateConfig) -> Self {
            let mut new_config = self.config;
            new_config.admin_auth = config.admin_auth.unwrap_or(new_config.admin_auth);
            new_config.band = config.band.unwrap_or(new_config.band);
            new_config.quote_symbol = config.quote_symbol.unwrap_or(new_config.quote_symbol);
            self.config = new_config;
            self
        }
    }
}
