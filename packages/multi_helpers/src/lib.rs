pub(crate) use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, ContractInfo, StdResult, Uint128, Uint256,
};
pub(crate) use serde::Serialize;
pub(crate) use shade_protocol::{
    multi_test::{App, AppResponse},
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable, Query},
    AnyResult,
};

#[cfg(not(target_arch = "wasm32"))]
pub mod multi;
#[cfg(not(target_arch = "wasm32"))]
pub use multi::*;
