#[cfg(not(target_arch = "wasm32"))]
pub mod contract_helpers;

#[cfg(test)]
mod test;

#[cfg(feature = "contracts")]
#[path = ""]
mod reexport_contracts {
    pub mod mock_band;
    pub mod proxy_band_oracle;
    pub mod earn_v1_oracle;
    pub mod lp_oracle;
    pub mod oracle_router;
}

#[cfg(feature = "contracts")]
pub use reexport_contracts::*;