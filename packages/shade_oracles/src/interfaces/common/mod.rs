use crate::interfaces::band::ReferenceData;
use crate::{BLOCK_SIZE, Query, InstantiateCallback, ExecuteCallback, ResponseStatus, Contract};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Uint256, StdError, StdResult};
/// Default Query API for all oracles.
///
/// Every oracle must support these 3 methods in addition to any specific ones it wants to support.
#[cw_serde]
pub enum OracleQuery {
    GetConfig {},
    GetPrice { key: String },
    GetPrices { keys: Vec<String> },
}

impl Query for OracleQuery {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum OracleExecuteMsg {
    UpdateConfig { 
        supported_keys: Option<Vec<String>>,
        symbols: Option<Vec<String>>,
        dependencies: Option<Vec<(String, Contract)>>,
        router: Option<Contract>,
        only_band: Option<bool>,
        enabled: Option<bool>,
    },
}

#[cw_serde]
pub struct CommonConfig {
    pub supported_keys: Vec<String>,
    pub symbols: Vec<String>,
    pub dependencies: Vec<(String, Contract)>,
    pub router: Contract,
    pub enabled: bool,
    pub only_band: bool,
}

impl ExecuteCallback for OracleExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

/// Default HandleAnswer for oracles if only ExecuteMsg implemented is UpdateConfig.
#[cw_serde]
pub enum HandleAnswer {
    UpdateConfig { status: ResponseStatus },
}

#[cw_serde]
pub struct OraclePrice {
    pub key: String,
    pub data: ReferenceData,
}

impl OraclePrice {
    pub fn new(key: String, reference_data: ReferenceData) -> Self {
        OraclePrice {
            key,
            data: reference_data,
        }
    }
}

pub fn throw_unsupported_symbol_error(key: String) -> StdError {
   StdError::generic_err(format!("{} is not supported as a key.", key))
}

pub fn is_disabled(enabled: bool) -> StdResult<()> {
    if !enabled {
        Err(StdError::generic_err("Deprecated oracle."))
    } else {
        Ok(())
    }
}

pub mod querier;