pub mod mocks {
    #[cfg(feature = "mock-dex")]
    pub mod dex {
        use mock_shade_pair;
        multi_derive::implement_multi!(MockShadePair, mock_shade_pair);

        use mock_sienna_pair;
        multi_derive::implement_multi!(MockSiennaPair, mock_sienna_pair);
    }
}

use admin;
multi_derive::implement_multi!(AdminAuth, admin);

use oracle_router;
multi_derive::implement_multi!(OracleRouter, oracle_router);

use proxy_band_oracle;
multi_derive::implement_multi!(ProxyBandOracle, proxy_band_oracle);

use mock_band;
multi_derive::implement_multi!(MockBand, mock_band);


