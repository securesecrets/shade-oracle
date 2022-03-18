use serde::{Deserialize, Serialize};
use shared_types::{
    asset::{CanonicalContract, Contract},
    scrt::{
        Api, CanonicalAddr, PrefixedStorage, ReadonlyPrefixedStorage, ReadonlyStorage, StdError,
        StdResult, Storage,
    },
    storage::{bincode_state::*, traits::SingletonStorable},
};

pub const KEY_CONFIG: &[u8] = b"YteGsgSZyO";
pub const KEY_ORACLES: &[u8] = b"d3a17d1b";

#[derive(Serialize, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfig {
    pub owner: CanonicalAddr,
}

impl SingletonStorable for RawConfig {
    fn namespace() -> Vec<u8> {
        KEY_CONFIG.to_vec()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Oracle {
    pub contract: CanonicalContract,
}

// Processors are keyed by (contract, token)
// The sender of the token is the contract whose fees need to be processed.
impl Oracle {
    pub fn new(api: &impl Api, contract: Contract) -> StdResult<Self> {
        Ok(Oracle {
            contract: contract.as_canonical(api)?,
        })
    }

    pub fn save<S: Storage>(self, storage: &mut S, key: &str) -> StdResult<()> {
        let mut store = PrefixedStorage::new(KEY_ORACLES, storage);
        save(&mut store, key.as_bytes(), &self)?;
        Ok(())
    }

    pub fn remove<S: Storage>(storage: &mut S, key: &str) -> StdResult<()> {
        let mut store = PrefixedStorage::new(KEY_ORACLES, storage);
        remove(&mut store, key.as_bytes());
        Ok(())
    }

    pub fn get<S: ReadonlyStorage>(storage: &S, api: &impl Api, key: &str) -> StdResult<Contract> {
        let store = ReadonlyPrefixedStorage::new(KEY_ORACLES, storage);
        let oracle: Option<Oracle> = may_load(&store, key.as_bytes())?;
        match oracle {
            Some(oracle) => Ok(oracle.contract.as_human(api)?),
            None => Err(StdError::generic_err(format!(
                "Could not find oraacle at {}.",
                key
            ))),
        }
    }
}
