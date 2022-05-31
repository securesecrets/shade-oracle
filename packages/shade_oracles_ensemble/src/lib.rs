#[cfg(not(target_arch = "wasm32"))]
pub mod harness;

/*
#[cfg(feature = "contracts")]
#[path = ""]
mod reexport_contracts {
    pub use ::mock_band;
    pub use ::oracle_router;
    pub use ::proxy_band_oracle;
    pub use ::siennaswap_lp_spot_oracle;
    pub use ::siennaswap_lp_oracle;
}

#[cfg(feature = "contracts")]
pub use reexport_contracts::*;
*/
