pub mod common;
pub use common::*;
pub mod interfaces;
pub mod protocols;

pub const BLOCK_SIZE: usize = 256;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    Addr, Api, MessageInfo, QuerierWrapper, StdError, StdResult, Storage, Timestamp, Uint128,
    Uint256,
};
use shade_protocol::utils::{
    asset::{Contract, RawContract},
    Query,
};

pub mod unit_test_interface;

#[cfg(feature = "storage")]
pub use storage::*;
#[cfg(feature = "storage")]
pub mod storage {
    pub use secret_storage_plus as ssp;
}
#[cfg(feature = "core")]
pub mod core {
    pub use better_secret_math;
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
            price::{get_precision, translate_price},
            ExecuteCallback, InstantiateCallback, Query,
        },
    };
    pub use thiserror;
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
