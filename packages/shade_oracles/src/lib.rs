pub const BLOCK_SIZE: usize = 256;

pub mod interfaces;
pub use interfaces::common as common;
pub mod protocols;

pub use shade_admin::admin::validate_admin;
pub use shade_protocol::utils::{Query, InstantiateCallback, ExecuteCallback, pad_handle_result, pad_query_result};
pub use shade_protocol::utils::generic_response::ResponseStatus;
pub use shade_protocol::utils::asset::{Contract, RawContract};
pub use shade_protocol::snip20;
pub use shade_protocol::utils::calc::{sqrt};
pub use shade_protocol::utils::price::{translate_price, normalize_price, get_precision};

#[cfg(feature = "storage")]
pub mod storage {
    pub use cosmwasm_storage::*;
    pub use shade_protocol::secret_storage_plus::*;
    pub use shade_protocol::utils::storage::plus::*;
    pub use shade_protocol::utils::storage::default::*;
}
#[cfg(feature = "core")]
pub mod core {
    pub use cosmwasm_schema;
    pub use schemars;
    pub use serde;
    pub use thiserror;
    #[cfg(feature = "scrt")]
    pub use {cosmwasm_std, cosmwasm_std::*};
}