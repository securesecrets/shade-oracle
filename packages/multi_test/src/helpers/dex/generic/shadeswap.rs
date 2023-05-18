#![allow(
    clippy::zero_prefixed_literal,
    clippy::inconsistent_digit_grouping,
    clippy::too_many_arguments
)]
use super::*;

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        harness::derivatives::shade, helpers::router::OracleRouterHelper, mocks::MockShadeswapPair,
    };
    use multi_test_helpers::Asserter;
    use oracle_mocks::shadeswap_pair::contract as mock_shade_pair;
    use shade_oracles::{
        interfaces::providers::RawProvider, unit_test_interface::prices::PricesFixture,
    };
    use shade_toolkit::{ExecuteCallback, InstantiateCallback};

    fn create_mock_lp_token(admin: &User, app: &mut App) -> (String, Snip20Helper) {
        let mock_liquidity_token_key = "MOCKLP";
        let liquidity_token = Snip20Helper::init(
            admin,
            app,
            mock_liquidity_token_key,
            mock_liquidity_token_key,
            6,
            &admin.address,
            &None,
            &to_binary("wdwdadwad").unwrap(),
            &format!("{mock_liquidity_token_key}-snip20"),
        );
        (mock_liquidity_token_key.to_string(), liquidity_token)
    }

    #[test]
    fn test_spot_oracle() {
        let TestScenario {
            mut app,
            router,
            admin,
            user,
            tokens,
            provider,
            ..
        } = TestScenario::new(PricesFixture::basic_prices_2());

        let shadeswap_oracle = GenericLiquidityPairOracleHelper::init_shadeswap_spot(
            &admin,
            &mut app,
            &router.clone().into(),
        );
        let shade_pair = mock_shade_pair::InstantiateMsg {}
            .test_init(
                MockShadeswapPair::default(),
                &mut app,
                admin.addr(),
                "shade_pair",
                &[],
            )
            .unwrap();

        let token_a_price = 10u128.pow(18);
        let token_b_price = 10u128.pow(18);
        let token_a_symbol = "FRAX";
        let token_b_symbol = "USDC";
        let new_prices = vec![("FRAX", token_a_price), ("USDC", token_b_price)];
        let (_, new_prices) = OracleCore::create_prices_hashmap(new_prices);
        provider.update_band_prices(&admin, &mut app, new_prices, None);

        let frax = tokens.get(token_a_symbol).unwrap();
        let usdc = tokens.get(token_b_symbol).unwrap();
        let frax_token_info = frax.query_token_info(&app).unwrap();
        let usdc_token_info = usdc.query_token_info(&app).unwrap();

        let pair_key = format!("{}-{}", token_a_symbol, token_b_symbol);

        let shade_pair_data = RawPairData {
            key: pair_key.clone(),
            base_token: frax.to_raw_asset(token_a_symbol),
            target_token: usdc.to_raw_asset(token_b_symbol),
            pair: shade_pair.clone().into(),
        };

        let (_, lp_token) = create_mock_lp_token(&admin, &mut app);
        let lp_token_info = lp_token.query_token_info(&app).unwrap();

        lp_token.add_minters(&mut app, &admin, vec![shade_pair.address.to_string()]);

        mock_shade_pair::ExecuteMsg::MockPool {
            token_a: frax.clone().into(),
            amount_a: Uint128::new(10u128.pow(frax_token_info.decimals as u32)),
            token_b: usdc.clone().into(),
            amount_b: Uint128::new(10u128.pow(usdc_token_info.decimals as u32)),
            liquidity_token: lp_token.clone().into(),
            liquidity_tokens: Uint128::new(10u128.pow(lp_token_info.decimals as u32)),
        }
        .test_exec(&shade_pair, &mut app, user.addr(), &[])
        .unwrap();

        assert!(shadeswap_oracle
            .set_pairs(&admin, &mut app, vec![shade_pair_data])
            .is_ok());
        assert!(router
            .set_keys(
                &admin,
                &mut app,
                shadeswap_oracle.0.clone().into(),
                vec![pair_key.clone()]
            )
            .is_ok());

        let lp_price = router.query_price(&app, pair_key).unwrap();
        assert_eq!(lp_price.data.rate, Uint256::from_u128(2 * 10u128.pow(18)));
    }

    #[test]
    fn test_registry() {
        let TestScenario {
            mut app,
            router,
            admin,
            user,
            tokens,
            keys,
            provider,
            admin_auth,
            ..
        } = TestScenario::new(PricesFixture::basic_prices_2());

        let new_router = OracleRouterHelper::init(
            &admin,
            &mut app,
            &admin_auth.into(),
            RawProvider::Band(provider.into()),
            "USD",
        );

        let shadeswap_oracles = vec![
            GenericLiquidityPairOracleHelper::init_shadeswap_spot(
                &admin,
                &mut app,
                &router.clone().into(),
            ),
            GenericLiquidityPairOracleHelper::init_shadeswap_market(
                &admin,
                &mut app,
                &router.into(),
            ),
        ];

        let shade_pair_a = mock_shade_pair::InstantiateMsg {}
            .test_init(
                MockShadeswapPair::default(),
                &mut app,
                admin.addr(),
                "shade_pair",
                &[],
            )
            .unwrap();

        let shade_pair_b = mock_shade_pair::InstantiateMsg {}
            .test_init(
                MockShadeswapPair::default(),
                &mut app,
                admin.addr(),
                "shade_pair_two",
                &[],
            )
            .unwrap();

        let frax = tokens.get(&keys[0]).unwrap();
        let usdc = tokens.get(&keys[1]).unwrap();
        let frax_token_info = frax.query_token_info(&app).unwrap();
        let usdc_token_info = usdc.query_token_info(&app).unwrap();

        let pair_a_key = format!("{}-{}", keys[0], keys[1]);
        let pair_b_key = format!("{}-{}", keys[1], keys[0]);

        let shade_pair_data_a = RawPairData {
            key: pair_a_key.clone(),
            base_token: frax.to_raw_asset(&keys[0]),
            target_token: usdc.to_raw_asset(&keys[1]),
            pair: shade_pair_a.clone().into(),
        };

        let shade_pair_data_b = RawPairData {
            key: pair_b_key.clone(),
            base_token: usdc.to_raw_asset(&keys[1]),
            target_token: frax.to_raw_asset(&keys[0]),
            pair: shade_pair_b.clone().into(),
        };

        let (_, lp_token) = create_mock_lp_token(&admin, &mut app);

        lp_token.add_minters(
            &mut app,
            &admin,
            vec![
                shade_pair_a.address.to_string(),
                shade_pair_b.address.to_string(),
            ],
        );

        mock_shade_pair::ExecuteMsg::MockPool {
            token_a: frax.clone().into(),
            amount_a: Uint128::new(100 * 10u128.pow(frax_token_info.decimals as u32)),
            token_b: usdc.clone().into(),
            amount_b: Uint128::new(100 * 10u128.pow(usdc_token_info.decimals as u32)),
            liquidity_token: lp_token.clone().into(),
            liquidity_tokens: Uint128::one(),
        }
        .test_exec(&shade_pair_a, &mut app, user.addr(), &[])
        .unwrap();

        mock_shade_pair::ExecuteMsg::MockPool {
            token_a: usdc.clone().into(),
            amount_a: Uint128::new(100 * 10u128.pow(usdc_token_info.decimals as u32)),
            token_b: frax.clone().into(),
            amount_b: Uint128::new(100 * 10u128.pow(frax_token_info.decimals as u32)),
            liquidity_token: lp_token.clone().into(),
            liquidity_tokens: Uint128::one(),
        }
        .test_exec(&shade_pair_b, &mut app, user.addr(), &[])
        .unwrap();

        for oracle in shadeswap_oracles {
            let original_router = oracle.query_config(&app).unwrap().config.router;

            // TEST STATUS UPDATE
            assert!(oracle.set_status(&user, &mut app, false).is_err());
            assert!(oracle.set_status(&admin, &mut app, false).is_ok());
            assert!(!oracle.query_config(&app).unwrap().config.enabled);
            assert!(oracle.set_status(&admin, &mut app, true).is_ok());
            assert!(oracle.query_config(&app).unwrap().config.enabled);
            assert_eq!(
                oracle.query_config(&app).unwrap().config.router,
                original_router
            );
            assert!(oracle.query_config(&app).unwrap().supported_keys.is_empty());

            let initial_pair_data = vec![shade_pair_data_a.clone(), shade_pair_data_b.clone()];

            // TEST CONFIG
            assert!(oracle
                .update_config(&user, &mut app, &new_router.clone().into())
                .is_err());
            assert!(oracle
                .update_config(&admin, &mut app, &new_router.clone().into())
                .is_ok());
            assert_eq!(
                oracle.query_config(&app).unwrap().config.router,
                new_router.clone().into()
            );
            assert!(oracle
                .update_config(&admin, &mut app, &original_router.clone().into())
                .is_ok());
            assert_eq!(
                oracle.query_config(&app).unwrap().config.router,
                original_router
            );

            // TEST SET PAIRS
            assert!(oracle
                .set_pairs(&user, &mut app, initial_pair_data.clone())
                .is_err());
            oracle
                .set_pairs(&admin, &mut app, initial_pair_data.clone())
                .unwrap();
            assert!(oracle
                .set_pairs(&admin, &mut app, initial_pair_data.clone())
                .is_ok());
            let initial_pairs = oracle.query_pairs(&app).unwrap();
            assert_eq!(initial_pairs.len(), initial_pair_data.len());

            for pair in initial_pairs {
                assert!(pair.key == pair_a_key || pair.key == pair_b_key);
                assert!(
                    pair.base_token == frax.to_asset(&app, &keys[0])
                        || pair.base_token == usdc.to_asset(&app, &keys[1])
                );
                assert!(
                    pair.target_token == frax.to_asset(&app, &keys[0])
                        || pair.target_token == usdc.to_asset(&app, &keys[1])
                );
                assert!(
                    pair.pair == shade_pair_a.clone().into()
                        || pair.pair == shade_pair_b.clone().into()
                );
            }

            Asserter::equal_vecs(
                &oracle.query_config(&app).unwrap().supported_keys,
                &[pair_a_key.clone(), pair_b_key.clone()],
            );

            // TEST UPDATE ASSETS
            assert!(oracle
                .update_assets(&user, &mut app, &[frax.to_raw_asset(&keys[1])])
                .is_err());
            assert!(oracle
                .update_assets(&admin, &mut app, &[frax.to_raw_asset(&keys[1])])
                .is_ok());
            let updated_pairs = oracle.query_pairs(&app).unwrap();
            assert_eq!(updated_pairs.len(), initial_pair_data.len());

            for pair in updated_pairs {
                assert!(pair.key == pair_a_key || pair.key == pair_b_key);
                assert!(
                    pair.base_token == frax.to_asset(&app, &keys[1])
                        || pair.base_token == usdc.to_asset(&app, &keys[1])
                );
                assert!(
                    pair.target_token == frax.to_asset(&app, &keys[1])
                        || pair.target_token == usdc.to_asset(&app, &keys[1])
                );
                assert!(
                    pair.pair == shade_pair_a.clone().into()
                        || pair.pair == shade_pair_b.clone().into()
                );
            }

            // TEST REMOVE PAIRS
            assert!(oracle
                .remove_pairs(&user, &mut app, &[pair_a_key.clone()])
                .is_err());
            assert!(oracle
                .remove_pairs(&admin, &mut app, &[pair_a_key.clone()])
                .is_ok());
            let updated_pairs = oracle.query_pairs(&app).unwrap();
            assert_eq!(updated_pairs.len(), initial_pair_data.len() - 1);
            assert!(updated_pairs.iter().all(|pair| pair.key != pair_a_key));
            assert!(oracle
                .query_config(&app)
                .unwrap()
                .supported_keys
                .iter()
                .all(|key| key != &pair_a_key));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn basic_market_test(
        symbol: String,
        _base_peg: Option<String>,
        prices: Vec<(&str, u128)>,
        primary_symbol: String,
        primary_pool: Uint128,
        primary_decimals: u32,
        base_symbol: String,
        base_pool: Uint128,
        base_decimals: u32,
        expected: Uint256,
    ) {
        let TestScenario {
            mut app,
            router,
            admin,
            ..
        } = TestScenario::new(prices);
        let user = admin.clone();

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

        let (_, lp_token) = create_mock_lp_token(&admin, &mut app);
        lp_token.add_minters(&mut app, &admin, vec![shade_pair.address.to_string()]);

        mock_shade_pair::ExecuteMsg::MockPool {
            token_a: primary_token.clone().into(),
            amount_a: primary_pool,
            token_b: base_token.clone().into(),
            amount_b: base_pool,
            liquidity_token: lp_token.clone().into(),
            liquidity_tokens: Uint128::one(),
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
            target_token: RawAsset::new(primary_token.0, primary_symbol.clone()),
            base_token: RawAsset::new(base_token.0, base_symbol.clone()),
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
                        Uint256::from_u128(expected)
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
