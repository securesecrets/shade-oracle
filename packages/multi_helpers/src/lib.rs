pub(crate) use cosmwasm_std::{to_binary, Binary, Coin, ContractInfo, StdResult, Uint128};

pub(crate) use shade_toolkit::{ExecuteCallback, InstantiateCallback, Query};

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
/// impl Into<shade_toolkit::Contract> for MyTestHelper {
///    fn into(self) -> shade_toolkit::Contract {
///       shade_toolkit::Contract {
///         address: self.0.address,
///        code_hash: self.0.code_hash,
///      }
///    }
/// }
/// impl Into<shade_toolkit::RawContract> for MyTestHelper {
///    fn into(self) -> shade_toolkit::RawContract {
///       shade_toolkit::RawContract {
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
        impl Into<shade_toolkit::Contract> for $name {
            fn into(self) -> shade_toolkit::Contract {
                shade_toolkit::Contract {
                    address: self.0.address,
                    code_hash: self.0.code_hash,
                }
            }
        }
        impl Into<shade_toolkit::RawContract> for $name {
            fn into(self) -> shade_toolkit::RawContract {
                shade_toolkit::RawContract {
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
        impl Into<shade_toolkit::Contract> for $name {
            fn into(self) -> shade_toolkit::Contract {
                shade_toolkit::Contract {
                    address: self.0.address,
                    code_hash: self.0.code_hash,
                }
            }
        }
        impl Into<shade_toolkit::RawContract> for $name {
            fn into(self) -> shade_toolkit::RawContract {
                shade_toolkit::RawContract {
                    address: self.0.address.to_string(),
                    code_hash: self.0.code_hash,
                }
            }
        }
    };
}
