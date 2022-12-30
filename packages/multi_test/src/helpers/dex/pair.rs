use super::*;
use crate::harness::dex::{ShadeSwapSpotOracle, SiennaSwapReservesOracle};
use shade_oracles::interfaces::dex::pair::*;

create_test_helper!(LiquidityPairOracleHelper);

impl LiquidityPairOracleHelper {
    pub fn init_shadeswap_spot(sender: &User, app: &mut App, router: &Contract) -> Self {
        let contract = sender
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                },
                ShadeSwapSpotOracle::default(),
                "shadeswap_spot_oracle",
            )
            .unwrap();
        Self(contract)
    }
    pub fn init_siennaswap_reserves(sender: &User, app: &mut App, router: &Contract) -> Self {
        let contract = sender
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                },
                SiennaSwapReservesOracle::default(),
                "siennaswap_reserves_oracle",
            )
            .unwrap();
        Self(contract)
    }
}
