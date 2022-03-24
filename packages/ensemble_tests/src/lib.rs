#[cfg(not(target_arch = "wasm32"))]
pub mod constants;

#[cfg(not(target_arch = "wasm32"))]
pub mod contract_helpers;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod test;
