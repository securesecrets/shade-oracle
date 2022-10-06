pub mod interfaces;
pub use interfaces::common;
pub const BLOCK_SIZE: usize = 256;
pub mod protocols;

#[cfg(test)]
pub mod unit_test_interface;

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
    pub use schemars;
    pub use secret_storage_plus as ssp;
    pub use serde;
    pub use shade_protocol::{
        admin, snip20,
        utils::{
            asset::{Contract, RawContract},
            calc::sqrt,
            generic_response::ResponseStatus,
            pad_handle_result, pad_query_result,
            price::{get_precision, normalize_price, translate_price},
            ExecuteCallback, InstantiateCallback, Query,
        },
    };
    pub use thiserror;
    #[cfg(feature = "scrt")]
    pub use {cosmwasm_std, cosmwasm_std::*};
}

#[macro_use]
pub extern crate better_secret_math;

#[macro_export]
macro_rules! create_attr_action {
    ($y:literal) => {
        #[macro_export]
        macro_rules! attr_action {
            ($x:literal) => {
                cosmwasm_std::attr("action", concat!($y, $x))
            };
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! impl_msg_callbacks {
    () => {
        impl shade_protocol::utils::InstantiateCallback for InstantiateMsg {
            const BLOCK_SIZE: usize = shade_protocol::BLOCK_SIZE;
        }

        impl shade_protocol::utils::ExecuteCallback for ExecuteMsg {
            const BLOCK_SIZE: usize = shade_protocol::BLOCK_SIZE;
        }

        impl shade_protocol::utils::Query for QueryMsg {
            const BLOCK_SIZE: usize = shade_protocol::BLOCK_SIZE;
        }
    };
}
