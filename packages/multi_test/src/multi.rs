pub mod mocks {
    pub mod shade_pair {
        use mock_shade_pair;
        multi_derive::implement_multi!(MockShadePair, mock_shade_pair);
    }
    pub mod sienna_pair {
        use mock_sienna_pair;
        multi_derive::implement_multi!(MockSiennaPair, mock_sienna_pair);
    }
}

pub use crate::multi::{
    {
        router::OracleRouter,
        band::{
            mock::MockBand,
            proxy::ProxyBandOracle,
        },
        mocks::{
            shade_pair::MockShadePair,
            sienna_pair::MockSiennaPair,
        }
    }
};

pub mod router
{
    use oracle_router;
    multi_derive::implement_multi!(OracleRouter, oracle_router);
    
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

