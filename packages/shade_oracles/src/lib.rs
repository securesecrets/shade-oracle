pub mod interfaces;
pub use interfaces::common;
pub const BLOCK_SIZE: usize = 256;
pub mod protocols;

#[cfg(feature = "storage")]
pub use storage::*;
#[cfg(feature = "storage")]
pub mod storage {
    pub use cosmwasm_storage::*;
    pub use secret_storage_plus as ssp;
}
#[cfg(feature = "core")]
pub mod core {
    pub use better_secret_math;
    pub use cosmwasm_schema;
    pub use mulberry;
    pub use schemars;
    pub use secret_storage_plus as ssp;
    pub use serde;
    pub use shade_admin::querier::validate_permission;
    pub use shade_protocol::snip20;
    pub use shade_protocol::utils::asset::{Contract, RawContract};
    pub use shade_protocol::utils::calc::sqrt;
    pub use shade_protocol::utils::generic_response::ResponseStatus;
    pub use shade_protocol::utils::price::{get_precision, normalize_price, translate_price};
    pub use shade_protocol::utils::{
        pad_handle_result, pad_query_result, ExecuteCallback, InstantiateCallback, Query,
    };
    pub use thiserror;
    #[cfg(feature = "scrt")]
    pub use {cosmwasm_std, cosmwasm_std::*};
}

#[macro_use]
pub extern crate mulberry;
