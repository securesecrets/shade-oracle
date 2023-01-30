macro_rules! create_harness {
    ($mod_name:ident, $name:ident, $contract:ident) => {
        pub use $mod_name::*;
        pub mod $mod_name {
            multi_derive::implement_multi!($name, $contract);
        }
    };
}

create_harness!(band, MockBand, mock_band);
create_harness!(router, OracleRouter, oracle_router);
create_harness!(index, IndexOracle, index_oracle);

pub mod dex {
    pub mod siennaswap {
        create_harness!(spot, SiennaSwapSpotOracle, siennaswap_spot_oracle);
        create_harness!(
            reserves,
            SiennaSwapReservesOracle,
            siennaswap_reserves_oracle
        );
        create_harness!(market, SiennaSwapMarketOracle, siennaswap_market_oracle);
    }
    pub mod shadeswap {
        create_harness!(spot, ShadeSwapSpotOracle, shadeswap_spot_oracle);
        create_harness!(market, ShadeSwapMarketOracle, shadeswap_market_oracle);
    }
}

pub mod derivatives {
    pub use shade::*;
    pub use stride::*;

    create_harness!(
        shade,
        ShadeStakingDerivativesOracle,
        shade_staking_derivatives_oracle
    );
    create_harness!(
        stride,
        StrideStakingDerivativesOracle,
        stride_staking_derivatives_oracle
    );
}
