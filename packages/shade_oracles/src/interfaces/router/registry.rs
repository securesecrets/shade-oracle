use cosmwasm_schema::cw_serde;
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
    Remove { key: String },
    Replace { oracle: Contract, key: String },
    Add { oracle: Contract, key: String },
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

pub struct Oracle;

#[cfg(feature = "router")]
pub use state::*;
#[cfg(feature = "router")]
mod state {
    use std::collections::HashMap;

    use crate::{
        impl_global_status,
        interfaces::router::{
            error::{OracleRouterError, OracleRouterResult},
            msg::KeysResponse,
        },
    };

    use super::*;
    use cosmwasm_std::{to_binary, Binary, Deps, StdError, StdResult, Storage};
    use secret_storage_plus::{GenericMapStorage, Item, ItemStorage, Map};

    impl_global_status!(OracleRouter, OracleRouterError);

    impl ItemStorage for Config {
        const ITEM: Item<'static, Self> = Item::new("oraclerouterconfig");
    }

    impl<'a> GenericMapStorage<'a, &'a str, Contract> for Oracle {
        const MAP: Map<'static, &'a str, Contract> = Map::new("oraclerouteroracleregistry");
    }

    pub const KEYS: Item<Vec<String>> = Item::new("oraclerouterkeys");

    impl OracleRouter {
        pub fn load(storage: &dyn Storage) -> OracleRouterResult<Self> {
            let config = Config::load(storage)?;
            Ok(OracleRouter { config })
        }

        pub fn get_keys(deps: Deps) -> OracleRouterResult<Binary> {
            let keys = KEYS.load(deps.storage)?;
            Ok(to_binary(&KeysResponse { keys })?)
        }

        pub fn resolve_registry_operation(
            storage: &mut dyn Storage,
            operation: RegistryOperation,
        ) -> OracleRouterResult<()> {
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
            }
            Ok(())
        }

        pub fn get_oracle(&self, storage: &dyn Storage, key: &str) -> OracleRouterResult<Contract> {
            match Oracle::may_load(storage, key)? {
                Some(contract) => Ok(contract),
                None => Ok(self.config.this.clone()),
            }
        }

        pub fn group_keys_by_oracle(
            &self,
            storage: &dyn Storage,
            keys: &[String],
        ) -> OracleRouterResult<HashMap<Contract, Vec<String>>> {
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
