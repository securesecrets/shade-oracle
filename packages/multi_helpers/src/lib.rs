pub(crate) use cosmwasm_std::{to_binary, Binary, Coin, ContractInfo, StdResult, Uint128};
pub(crate) use serde::Serialize;
pub(crate) use shade_protocol::utils::{ExecuteCallback, InstantiateCallback, Query};

#[cfg(not(target_arch = "wasm32"))]
mod helpers;
#[cfg(not(target_arch = "wasm32"))]
pub use helpers::*;

/// Macro to generate the base for a test helper struct.
///
/// Example:
/// impl_test_helper!(MyTestHelper);
///
/// This will generate a struct with the following:
/// ```
/// #[derive(Clone)]
/// pub struct MyTestHelper(pub cosmwasm_std::ContractInfo);
/// impl Into<shade_protocol::utils::asset::Contract> for MyTestHelper {
///    fn into(self) -> shade_protocol::utils::asset::Contract {
///       shade_protocol::utils::asset::Contract {
///         address: self.0.address,
///        code_hash: self.0.code_hash,
///      }
///    }
/// }
/// impl Into<shade_protocol::utils::asset::RawContract> for MyTestHelper {
///    fn into(self) -> shade_protocol::utils::asset::RawContract {
///       shade_protocol::utils::asset::RawContract {
///         address: self.0.address.to_string(),
///         code_hash: self.0.code_hash,
///       }
///    }
/// }
/// ```
#[macro_export]
macro_rules! create_test_helper {
    ($name:ident) => {
        #[derive(Clone)]
        pub struct $name(pub cosmwasm_std::ContractInfo);
        impl Into<shade_protocol::utils::asset::Contract> for $name {
            fn into(self) -> shade_protocol::utils::asset::Contract {
                shade_protocol::utils::asset::Contract {
                    address: self.0.address,
                    code_hash: self.0.code_hash,
                }
            }
        }
        impl Into<shade_protocol::utils::asset::RawContract> for $name {
            fn into(self) -> shade_protocol::utils::asset::RawContract {
                shade_protocol::utils::asset::RawContract {
                    address: self.0.address.to_string(),
                    code_hash: self.0.code_hash,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! create_test_helper_with_user {
    ($name:ident) => {
        #[derive(Clone)]
        pub struct $name(pub cosmwasm_std::ContractInfo, pub multi_test_helpers::User);
        impl Into<shade_protocol::utils::asset::Contract> for $name {
            fn into(self) -> shade_protocol::utils::asset::Contract {
                shade_protocol::utils::asset::Contract {
                    address: self.0.address,
                    code_hash: self.0.code_hash,
                }
            }
        }
        impl Into<shade_protocol::utils::asset::RawContract> for $name {
            fn into(self) -> shade_protocol::utils::asset::RawContract {
                shade_protocol::utils::asset::RawContract {
                    address: self.0.address.to_string(),
                    code_hash: self.0.code_hash,
                }
            }
        }
    };
}
