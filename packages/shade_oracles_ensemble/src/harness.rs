#[cfg(feature = "siennaswap-spot-lp")]
pub mod siennaswap_spot_lp {
    use ensemble_harness::helpers;
    use siennaswap_lp_spot_oracle;

    pub struct SiennaSwapLpOracle;
    helpers::implement_harness!(SiennaSwapLpOracle, siennaswap_lp_spot_oracle);
}

#[cfg(feature = "siennaswap-lp")]
pub mod siennaswap_lp {
    use ensemble_harness::helpers;
    use siennaswap_lp_oracle;

    pub struct SiennaSwapLp;
    helpers::implement_harness!(SiennaSwapLp, siennaswap_lp_oracle);
}

#[cfg(feature = "band-mock")]
pub mod band_mock {
    use ensemble_harness::helpers;
    use mock_band;
    pub struct MockBand;
    helpers::implement_harness!(MockBand, mock_band);
}

#[cfg(feature = "proxy-band")]
pub mod proxy_band {
    use ensemble_harness::helpers;
    use proxy_band_oracle;

    pub struct ProxyBandOracle;
    helpers::implement_harness!(ProxyBandOracle, proxy_band_oracle);
}

#[cfg(feature = "router")]
pub mod router {
    use ensemble_harness::helpers;
    use oracle_router;

    pub struct OracleRouter;
    helpers::implement_harness!(OracleRouter, oracle_router);
}