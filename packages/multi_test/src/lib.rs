#[cfg(not(target_arch = "wasm32"))]
pub mod multi;
#[cfg(not(target_arch = "wasm32"))]
pub mod helpers;
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "multi-test")]
pub use shade_protocol::multi_test::*;
pub use shade_protocol::utils::MultiTestable;