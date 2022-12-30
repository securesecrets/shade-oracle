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

pub use crate::{
    band::mock::MockBand,
    mocks::{shade_pair::MockShadePair, sienna_pair::MockSiennaPair},
    router::OracleRouter,
};

pub mod router {
    use oracle_router;
    multi_derive::implement_multi!(OracleRouter, oracle_router);
    pub use shade_oracles::interfaces::router::*;
}

pub mod band {
    pub mod mock {
        use mock_band;
        multi_derive::implement_multi!(MockBand, mock_band);
    }
}

pub mod index {
    use index_oracle;
    multi_derive::implement_multi!(IndexOracle, index_oracle);
}

pub mod dex {
    pub use a::*;
    pub use b::*;
    pub use c::*;
    pub use d::*;
    pub use e::*;

    mod a {
        multi_derive::implement_multi!(SiennaSwapSpotOracle, siennaswap_spot_oracle);
    }

    mod b {
        multi_derive::implement_multi!(SiennaSwapReservesOracle, siennaswap_reserves_oracle);
    }

    mod c {
        multi_derive::implement_multi!(SiennaSwapMarketOracle, siennaswap_market_oracle);
    }

    mod d {
        multi_derive::implement_multi!(ShadeSwapMarketOracle, shadeswap_market_oracle);
    }

    mod e {
        multi_derive::implement_multi!(ShadeSwapSpotOracle, shadeswap_spot_oracle);
    }
}

pub mod derivatives {
    pub use a::*;
    pub use b::*;

    mod a {
        multi_derive::implement_multi!(
            ShadeStakingDerivativesOracle,
            shade_staking_derivatives_oracle
        );
    }

    mod b {
        multi_derive::implement_multi!(
            StrideStakingDerivativesOracle,
            stride_staking_derivatives_oracle
        );
    }
}
