use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use shade_protocol::Contract;

#[cw_serde]
pub struct Config {
    pub address: Addr,
    pub admin_auth: Contract,
    pub default_oracle: Contract,
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
    pub default_oracle: Option<Contract>,
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
        interfaces::router::error::{OracleRouterError, OracleRouterResult},
    };

    use super::*;
    use cosmwasm_std::Storage;
    use secret_storage_plus::{GenericMapStorage, Item, ItemStorage, Map};

    impl_global_status!(OracleRouter, OracleRouterError);

    impl ItemStorage for Config {
        const ITEM: Item<'static, Self> = Item::new("oraclerouterconfig");
    }

    impl<'a> GenericMapStorage<'a, &'a str, Contract> for Oracle {
        const MAP: Map<'static, &'a str, Contract> = Map::new("oraclerouteroracleregistry");
    }

    impl OracleRouter {
        pub fn load(storage: &dyn Storage) -> OracleRouterResult<Self> {
            let config = Config::load(storage)?;
            Ok(OracleRouter { config })
        }

        pub fn resolve_registry_operation(
            storage: &mut dyn Storage,
            operation: RegistryOperation,
        ) -> OracleRouterResult<()> {
            match operation {
                RegistryOperation::Remove { key } => {
                    Oracle::MAP.remove(storage, &key);
                }
                RegistryOperation::Replace { oracle, key } => {
                    Oracle::save(storage, &key, &oracle)?;
                }
                RegistryOperation::Add { oracle, key } => {
                    Oracle::MAP.update(storage, &key, |old_oracle| -> OracleRouterResult<_> {
                        match old_oracle {
                            Some(_) => Err(OracleRouterError::ExistingOracle {
                                key: key.to_string(),
                            }),
                            None => Ok(oracle),
                        }
                    })?;
                }
            }
            Ok(())
        }

        pub fn get_oracle(&self, storage: &dyn Storage, key: &str) -> OracleRouterResult<Contract> {
            match Oracle::may_load(storage, key)? {
                Some(contract) => Ok(contract),
                None => Ok(self.config.default_oracle.clone()),
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
            new_config.default_oracle = config.default_oracle.unwrap_or(new_config.default_oracle);
            new_config.band = config.band.unwrap_or(new_config.band);
            new_config.quote_symbol = config.quote_symbol.unwrap_or(new_config.quote_symbol);
            self.config = new_config;
            self
        }
    }
}
