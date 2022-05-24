use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::{CanonicalContract, Contract},
    scrt::{
        Api, CanonicalAddr, StdResult,
    },
};

pub const KEY_CONFIG: &[u8] = b"YteGsgSZyO";
pub const KEY_ORACLES: &[u8] = b"d3a17d1b";

#[derive(Serialize, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfig {
    pub owner: CanonicalAddr,
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
}
