use super::*;

        let market_oracle = shadeswap_market_oracle::InstantiateMsg {
            config: InstantiateCommonConfig {
                supported_keys: None,
                router: router.clone().into(),
                enabled: true,
                only_band: true,
            },
            base_peg,
            symbol: symbol.clone(),
            pair: shade_pair.clone().into(),
        }
        .test_init(
            ShadeSwapMarketOracle::default(),
            &mut app,
            user.clone(),
            "shade-swap-market-oracle",
            &[],
        )
        .unwrap();