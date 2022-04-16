#[cfg(not(target_arch = "wasm32"))]
pub mod contract_helpers;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod test;

#[cfg(feature = "contracts")]
#[path = ""]
mod reexport_contracts {
    pub use ::earn_v1_oracle;
    pub use ::mock_band;
    pub use ::oracle_router;
    pub use ::proxy_band_oracle;
    pub use ::secretswap_lp_oracle;
    pub use ::siennaswap_lp_oracle;
}

#[cfg(feature = "contracts")]
pub use reexport_contracts::*;
