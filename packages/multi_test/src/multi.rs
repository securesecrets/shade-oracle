pub mod mocks {
    pub mod shade_pair {
        use mock_shade_pair;
        multi_derive::implement_multi!(MockShadePair, mock_shade_pair);
    }
    pub mod sienna_pair {
        use mock_sienna_pair;
        multi_derive::implement_multi!(MockSiennaPair, mock_sienna_pair);
    }
    #[cfg(feature = "snip20")]
    pub use shade_multi_test::multi::snip20::Snip20;
}

pub use crate::multi::{
    band::{mock::MockBand, proxy::ProxyBandOracle},
    mocks::{shade_pair::MockShadePair, sienna_pair::MockSiennaPair},
    router::OracleRouter,
};

pub mod router {
    use oracle_router;
    multi_derive::implement_multi!(OracleRouter, oracle_router);
    pub use shade_oracles::interfaces::router::*;
}

pub mod band {
    pub mod proxy {
        use proxy_band_oracle;
        multi_derive::implement_multi!(ProxyBandOracle, proxy_band_oracle);
    }
    pub mod mock {
        use mock_band;
        multi_derive::implement_multi!(MockBand, mock_band);
    }
}

#[cfg(feature = "index-oracle")]
pub mod index {
    use index_oracle;
    multi_derive::implement_multi!(IndexOracle, index_oracle);
}

#[cfg(feature = "market")]
pub mod market {
    pub mod siennaswap {
        multi_derive::implement_multi!(SiennaSwapMarketOracle, siennaswap_market_oracle);
    }
    pub mod shadeswap {
        multi_derive::implement_multi!(ShadeSwapMarketOracle, shadeswap_market_oracle);
    }
}

#[cfg(feature = "lp")]
pub mod lp {
    pub mod spot {
        multi_derive::implement_multi!(SiennaSwapLpSpotOracle, siennaswap_lp_spot_oracle);
    }
    multi_derive::implement_multi!(SiennaSwapLpOracle, siennaswap_lp_oracle);
}

#[cfg(feature = "staking-derivative")]
pub mod staking_derivative {
    multi_derive::implement_multi!(
        ShadeStakingDerivativeOracle,
        shade_staking_derivative_oracle
    );
}
