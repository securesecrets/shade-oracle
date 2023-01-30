use super::*;
use crate::harness::dex::{
    shadeswap::{ShadeSwapMarketOracle, ShadeSwapSpotOracle},
    siennaswap::{SiennaSwapMarketOracle, SiennaSwapReservesOracle, SiennaSwapSpotOracle},
};
use shade_oracles::interfaces::{common::config::CommonConfigResponse, dex::generic::*};

create_test_helper!(GenericLiquidityPairOracleHelper);

impl GenericLiquidityPairOracleHelper {
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

    pub fn init_siennaswap_spot(sender: &User, app: &mut App, router: &Contract) -> Self {
        let contract = sender
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                },
                SiennaSwapSpotOracle::default(),
                "siennaswap_spot_oracle",
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

    pub fn init_shadeswap_market(sender: &User, app: &mut App, router: &Contract) -> Self {
        let contract = sender
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                },
                ShadeSwapMarketOracle::default(),
                "shadeswap_market_oracle",
            )
            .unwrap();
        Self(contract)
    }

    pub fn init_siennaswap_market(sender: &User, app: &mut App, router: &Contract) -> Self {
        let contract = sender
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                },
                SiennaSwapMarketOracle::default(),
                "siennaswap_market_oracle",
            )
            .unwrap();
        Self(contract)
    }

    pub fn set_pairs(
        &self,
        sender: &User,
        app: &mut App,
        data: Vec<RawPairData>,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::SetPairs(data), &self.0)
    }

    pub fn remove_pairs(
        &self,
        sender: &User,
        app: &mut App,
        keys: &[String],
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::RemovePairs(keys.to_vec()), &self.0)
    }

    pub fn update_assets(
        &self,
        sender: &User,
        app: &mut App,
        assets: &[RawAsset],
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::UpdateAssets(assets.to_vec()), &self.0)
    }

    pub fn update_config(
        &self,
        sender: &User,
        app: &mut App,
        router: &RawContract,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::UpdateConfig(router.clone()), &self.0)
    }

    pub fn set_status(&self, sender: &User, app: &mut App, status: bool) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::SetStatus(status), &self.0)
    }

    pub fn query_pairs(&self, app: &App) -> StdResult<PairsResponse> {
        QueryMsg::GetPairs {}.test_query(&self.0, app)
    }
    pub fn query_config(&self, app: &App) -> StdResult<CommonConfigResponse> {
        QueryMsg::GetConfig {}.test_query(&self.0, app)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    mod shadeswap {
        #![allow(
            clippy::zero_prefixed_literal,
            clippy::inconsistent_digit_grouping,
            clippy::too_many_arguments
        )]

        use super::*;
        use crate::mocks::MockShadeswapPair;
        use oracle_mocks::shadeswap_pair::contract as mock_shade_pair;
        use shade_oracles::unit_test_interface::prices::PricesFixture;

        #[test]
        fn test_registry() {
            let TestScenario {
                mut app,
                router,
                admin,
                tokens,
                ..
            } = TestScenario::new(PricesFixture::basic_prices_2());
            let user = admin;

            let shadeswap_oracles = vec![
                GenericLiquidityPairOracleHelper::init_shadeswap_spot(
                    &user,
                    &mut app,
                    &router.clone().into(),
                ),
                GenericLiquidityPairOracleHelper::init_shadeswap_market(
                    &user,
                    &mut app,
                    &router.clone().into(),
                ),
            ];

            let shade_pair = mock_shade_pair::InstantiateMsg {}
                .test_init(
                    MockShadeswapPair::default(),
                    &mut app,
                    user.addr(),
                    "shade_pair",
                    &[],
                )
                .unwrap();

            let shade_pair_2 = mock_shade_pair::InstantiateMsg {}
                .test_init(
                    MockShadeswapPair::default(),
                    &mut app,
                    user.addr(),
                    "shade_pair_two",
                    &[],
                )
                .unwrap();

            // mock_shade_pair::ExecuteMsg::MockPool {
            //     token_a: primary_token.clone().into(),
            //     amount_a: primary_pool,
            //     token_b: base_token.clone().into(),
            //     amount_b: base_pool,
            // }
            // .test_exec(&shade_pair, &mut app, user.addr(), &[])
            // .unwrap();

            // let pair = RawPairData {
            //     target_token: RawAsset::new(primary_token.0.clone(), primary_symbol.clone()),
            //     base_token: RawAsset::new(base_token.0.clone(), base_symbol.clone()),
            //     key: symbol.clone(),
            //     pair: shade_pair.clone().into(),
            // };

            // market_oracle
            //     .set_pairs(&user, &mut app, vec![pair])
            //     .unwrap();

            // router
            //     .set_keys(
            //         &user,
            //         &mut app,
            //         market_oracle.clone().0.into(),
            //         vec![symbol.clone()],
            //     )
            //     .unwrap();
        }

        #[allow(clippy::too_many_arguments)]
        fn basic_market_test(
            symbol: String,
            base_peg: Option<String>,
            prices: Vec<(&str, u128)>,
            primary_symbol: String,
            primary_pool: Uint128,
            primary_decimals: u32,
            base_symbol: String,
            base_pool: Uint128,
            base_decimals: u32,
            expected: Uint128,
        ) {
            let TestScenario {
                mut app,
                router,
                admin,
                ..
            } = TestScenario::new(prices);
            let user = admin;

            // Setup tokens
            let primary_token = Snip20Helper::init(
                &user,
                &mut app,
                "Primary",
                primary_symbol.as_str(),
                primary_decimals as u8,
                &user.addr(),
                &None,
                &to_binary("").ok().unwrap(),
                "primary_token",
            );

            let base_token = Snip20Helper::init(
                &user,
                &mut app,
                "Base",
                base_symbol.as_str(),
                base_decimals as u8,
                &user.addr(),
                &None,
                &to_binary("").ok().unwrap(),
                "base_token",
            );

            let shade_pair = mock_shade_pair::InstantiateMsg {}
                .test_init(
                    MockShadeswapPair::default(),
                    &mut app,
                    user.addr(),
                    "shade_pair",
                    &[],
                )
                .unwrap();

            mock_shade_pair::ExecuteMsg::MockPool {
                token_a: primary_token.clone().into(),
                amount_a: primary_pool,
                token_b: base_token.clone().into(),
                amount_b: base_pool,
            }
            .test_exec(&shade_pair, &mut app, user.addr(), &[])
            .unwrap();

            // Configure router w/ market oracle

            let market_oracle = GenericLiquidityPairOracleHelper::init_shadeswap_market(
                &user,
                &mut app,
                &router.clone().into(),
            );

            let pair = RawPairData {
                target_token: RawAsset::new(primary_token.0.clone(), primary_symbol.clone()),
                base_token: RawAsset::new(base_token.0.clone(), base_symbol.clone()),
                key: symbol.clone(),
                pair: shade_pair.clone().into(),
            };

            market_oracle
                .set_pairs(&user, &mut app, vec![pair])
                .unwrap();

            router
                .set_keys(
                    &user,
                    &mut app,
                    market_oracle.clone().0.into(),
                    vec![symbol.clone()],
                )
                .unwrap();

            let pairs = market_oracle.query_pairs(&app).unwrap();
            assert!(pairs.len() == 1);
            assert_eq!(pairs[0].key, symbol);
            assert_eq!(pairs[0].target_token.quote_symbol, primary_symbol);
            assert_eq!(pairs[0].base_token.quote_symbol, base_symbol);

            let price: PriceResponse = router.query_price(&app, symbol).unwrap();

            let data = price.data();
            assert_eq!(
                expected, data.rate,
                "Expected: {} Got: {}",
                expected, data.rate
            );
        }

        macro_rules! basic_market_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (symbol, base_peg, prices,
                         primary_symbol, primary_pool, primary_decimals,
                         base_symbol, base_pool, base_decimals, expected) = $value;

                    basic_market_test(
                        symbol.to_string(),
                        base_peg,
                        prices,
                        primary_symbol.to_string(),
                        Uint128::from(primary_pool),
                        primary_decimals,
                        base_symbol.to_string(),
                        Uint128::from(base_pool),
                        base_decimals,
                        Uint128::from(expected)
                    );
                }
            )*
        }
    }

        basic_market_tests! {
            usd_stables_same_decimals: (
                "USDT",
                None,
                vec![
                    ("USDT", 10u128.pow(18)), // $1
                    ("USDC", 10u128.pow(18)), // $1
                ],
                "USDT",
                10u128.pow(10),
                6,
                "USDC",
                10u128.pow(10),
                6,
                999901 * 10u128.pow(12), // ~$.99 (slippage)
            ),
            usd_stables_diff_decimals: (
                "USDT",
                None,
                vec![
                    ("USDT", 10u128.pow(18)), // $1
                    ("USDC", 10u128.pow(18)), // $1
                ],
                "USDT",
                10u128.pow(16),
                13,
                "USDC",
                10u128.pow(21),
                18,
                999000999000999001u128, // ~$.99 (slippage)
            ),
            shd_sscrt_mainnet_test: (
                "SHD",
                None,
                vec![
                    ("SSCRT", 979575 * 10u128.pow(12)), // $0.979575
                ],

                "SHD",
                1_025_322 * 10u128.pow(6),
                8,

                "SSCRT",
                8_677_013 * 10u128.pow(4),
                6,

                8_289_060_794_625 * 10u128.pow(6), // ~$8.28
            ),
        }
    }

    mod siennaswap {
        #![allow(
            clippy::zero_prefixed_literal,
            clippy::inconsistent_digit_grouping,
            clippy::too_many_arguments
        )]
        use super::*;
        use crate::mocks::MockSiennaPair;
        use oracle_mocks::sienna_pair::contract as mock_sienna_pair;

        #[allow(clippy::too_many_arguments)]
        fn basic_market_test(
            symbol: String,
            base_peg: Option<String>,
            prices: Vec<(&str, u128)>,
            primary_symbol: String,
            primary_pool: Uint128,
            primary_decimals: u32,
            base_symbol: String,
            base_pool: Uint128,
            base_decimals: u32,
            expected: Uint128,
        ) {
            let TestScenario {
                mut app,
                router,
                admin,
                ..
            } = TestScenario::new(prices);
            let user = admin;

            // Setup tokens
            let primary_token = Snip20Helper::init(
                &user,
                &mut app,
                "Primary",
                primary_symbol.as_str(),
                primary_decimals as u8,
                &user.addr(),
                &None,
                &to_binary("").ok().unwrap(),
                "primary_token",
            );

            let base_token = Snip20Helper::init(
                &user,
                &mut app,
                "Base",
                base_symbol.as_str(),
                base_decimals as u8,
                &user.addr(),
                &None,
                &to_binary("").ok().unwrap(),
                "base_token",
            );

            let siennaswap_pair = mock_sienna_pair::InstantiateMsg {}
                .test_init(
                    MockSiennaPair::default(),
                    &mut app,
                    user.addr(),
                    "shade_pair",
                    &[],
                )
                .unwrap();

            mock_sienna_pair::ExecuteMsg::MockPool {
                token_a: primary_token.clone().into(),
                amount_a: primary_pool,
                token_b: base_token.clone().into(),
                amount_b: base_pool,
            }
            .test_exec(&siennaswap_pair, &mut app, user.addr(), &[])
            .unwrap();

            mock_sienna_pair::ExecuteMsg::MockPool {
                token_a: primary_token.clone().into(),
                amount_a: primary_pool,
                token_b: base_token.clone().into(),
                amount_b: base_pool,
            }
            .test_exec(&siennaswap_pair, &mut app, user.addr(), &[])
            .unwrap();

            let pair = RawPairData {
                target_token: RawAsset::new(primary_token.0.clone(), primary_symbol.clone()),
                base_token: RawAsset::new(base_token.0.clone(), base_symbol.clone()),
                key: symbol.clone(),
                pair: siennaswap_pair.clone().into(),
            };

            let market_oracle = GenericLiquidityPairOracleHelper::init_siennaswap_market(
                &user,
                &mut app,
                &router.clone().into(),
            );

            market_oracle
                .set_pairs(&user, &mut app, vec![pair])
                .unwrap();

            router
                .set_keys(
                    &user,
                    &mut app,
                    market_oracle.clone().0.into(),
                    vec![symbol.clone()],
                )
                .unwrap();

            let pairs = market_oracle.query_pairs(&app).unwrap();
            assert!(pairs.len() == 1);
            assert_eq!(pairs[0].key, symbol);
            assert_eq!(pairs[0].target_token.quote_symbol, primary_symbol);
            assert_eq!(pairs[0].base_token.quote_symbol, base_symbol);

            let price: PriceResponse = router.query_price(&app, symbol).unwrap();
            let data = price.data;

            assert_eq!(
                expected, data.rate,
                "Expected: {} Got: {}",
                expected, data.rate
            );
        }

        macro_rules! basic_market_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (symbol, base_peg, prices,
                         primary_symbol, primary_pool, primary_decimals,
                         base_symbol, base_pool, base_decimals, expected) = $value;

                    basic_market_test(
                        symbol.to_string(),
                        base_peg,
                        prices,
                        primary_symbol.to_string(),
                        Uint128::from(primary_pool),
                        primary_decimals,
                        base_symbol.to_string(),
                        Uint128::from(base_pool),
                        base_decimals,
                        Uint128::from(expected),
                    );
                }
            )*
        }
    }

        basic_market_tests! {
            usd_stables_same_decimals: (
                "USDT",
                None,
                vec![
                    ("USDT", 10u128.pow(18)), // $1
                    ("USDC", 10u128.pow(18)), // $1
                ],
                "USDT",
                10u128.pow(10),
                6,
                "USDC",
                10u128.pow(10),
                6,
                0_999901 * 10u128.pow(12), // ~$.99 (slippage)
            ),
            usd_stables_diff_decimals: (
                "USDT",
                None,
                vec![
                    ("USDT", 10u128.pow(18)), // $1
                    ("USDC", 10u128.pow(18)), // $1
                ],
                "USDT",
                10u128.pow(16),
                13,
                "USDC",
                10u128.pow(21),
                18,
                999000999000999001u128, // ~$.99 (slippage)
            ),
            shd_sscrt_mainnet_test: (
                "SHD",
                None,
                vec![
                    ("SSCRT", 979575 * 10u128.pow(12)), // $0.979575
                ],

                "SHD",
                10253_22 * 10u128.pow(6),
                8,

                "SSCRT",
                86770_13 * 10u128.pow(4),
                6,

                8_289_060_794_625 * 10u128.pow(6), // ~$8.28
            ),
        }
    }
}
