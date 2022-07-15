pub const BLOCK_SIZE: usize = 256;

pub mod interfaces;
pub use interfaces::common as common;
pub mod protocols;

pub use shade_protocol::utils::{Query, InstantiateCallback, ExecuteCallback};
pub use shade_protocol::utils::generic_response::ResponseStatus;
pub use shade_protocol::utils::asset::{Contract, UnvalidatedContract};

#[cfg(feature = "storage")]
pub use shade_protocol::secret_storage_plus as storage;
#[cfg(feature = "core")]
pub mod core {
    pub use cosmwasm_schema;
    pub use cosmwasm_storage;
    pub use schemars;
    pub use serde;
    pub use thiserror;
    #[cfg(feature = "scrt")]
    pub use {cosmwasm_std, cosmwasm_std::*};
}