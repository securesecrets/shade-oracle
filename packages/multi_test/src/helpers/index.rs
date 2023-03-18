use std::str::FromStr;

use super::*;
use crate::harness::index::IndexOracle;
use shade_oracles::{core::Query, interfaces::index::msg::*, status::ContractStatus};

create_test_helper!(IndexOracleHelper);
impl IndexOracleHelper {
    pub fn init(
        user: &User,
        app: &mut App,
        router: &Contract,
        basket: &[(String, Decimal256)],
        target: Uint256,
        symbol: &str,
        when_stale: u64,
        deviation_threshold: Decimal256,
    ) -> Self {
        let contract = user
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                    basket: basket.to_vec(),
                    target,
                    symbol: symbol.to_string(),
                    when_stale: Uint64::new(when_stale),
                    deviation_threshold,
                },
                IndexOracle::default(),
                "index_oracle",
            )
            .unwrap();
        Self(contract)
    }

    pub fn update_status(
        &self,
        sender: &User,
        app: &mut App,
        status: ContractStatus,
    ) -> AnyResult<AppResponse> {
        sender.exec(
            app,
            &ExecuteMsg::Admin(AdminMsg::UpdateStatus(status)),
            &self.0,
        )
    }

    pub fn mod_basket(
        &self,
        sender: &User,
        app: &mut App,
        basket: &[InitialBasketItem],
    ) -> AnyResult<AppResponse> {
        sender.exec(
            app,
            &ExecuteMsg::Admin(AdminMsg::ModBasket(basket.to_vec())),
            &self.0,
        )
    }

    pub fn update_config(
        &self,
        sender: &User,
        app: &mut App,
        symbol: Option<String>,
        router: Option<Contract>,
        when_stale: Option<u64>,
        deviation_threshold: Option<Decimal256>,
    ) -> AnyResult<AppResponse> {
        sender.exec(
            app,
            &ExecuteMsg::Admin(AdminMsg::UpdateConfig {
                symbol,
                router: router.map(|r| r.into()),
                when_stale: when_stale.map(Uint64::new),
                deviation_threshold,
            }),
            &self.0,
        )
    }

    pub fn update_target(
        &self,
        sender: &User,
        app: &mut App,
        target: Uint256,
    ) -> AnyResult<AppResponse> {
        sender.exec(
            app,
            &ExecuteMsg::Admin(AdminMsg::UpdateTarget(target)),
            &self.0,
        )
    }

    pub fn unfreeze(&self, sender: &User, app: &mut App) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::Admin(AdminMsg::UnfreezePeg {}), &self.0)
    }

    pub fn compute_index(&self, sender: &User, app: &mut App) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::ComputeIndex {}, &self.0)
    }

    pub fn query_basket(&self, app: &App) -> StdResult<BasketResponse> {
        QueryMsg::GetBasket {}.test_query(&self.0, app)
    }

    pub fn query_index_data(&self, app: &App) -> StdResult<IndexDataResponse> {
        QueryMsg::GetIndexData {}.test_query(&self.0, app)
    }

    // HELPER FUNCTIONS

    pub fn create_basket(
        prices: Vec<(impl Into<String> + Clone, impl Into<String> + Clone)>,
    ) -> Vec<InitialBasketItem> {
        prices
            .into_iter()
            .map(|(sym, w)| {
                (
                    sym.into().to_string(),
                    Decimal256::from_str(&w.into()).unwrap(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use shade_oracles::{
        better_secret_math::asserter::MathAsserter,
        interfaces::{common::OraclePrice, index::SIX_HOURS},
        unit_test_interface::prices::generate_price_feed,
    };
    use std::{convert::TryInto, str::FromStr};

    fn usd_basket() -> Vec<InitialBasketItem> {
        vec![
            ("USD".into(), Decimal256::percent(100)),
            ("EURO".into(), Decimal256::percent(0)),
            ("GDP".into(), Decimal256::percent(0)),
            ("JPY".into(), Decimal256::percent(0)),
        ]
    }

    fn basic_basket() -> Vec<InitialBasketItem> {
        vec![
            ("USD".into(), Decimal256::percent(25)),
            ("EURO".into(), Decimal256::percent(25)),
            ("GDP".into(), Decimal256::percent(25)),
            ("JPY".into(), Decimal256::percent(25)),
        ]
    }

    fn feed_0() -> Vec<OraclePrice> {
        generate_price_feed(vec![
            ("USD", "1.00", 0),
            ("EURO", "1.0196", 0),
            ("GDP", "1.208", 0),
            ("JPY", "0.0074", 0),
        ])
    }

    fn feed_2() -> Vec<OraclePrice> {
        generate_price_feed(vec![
            ("USD", "1.00", 0),
            ("EURO", "0.0196", 0),
            ("GDP", "1.208", 0),
            ("JPY", "0.0074", 0),
        ])
    }

    fn feed_3() -> Vec<OraclePrice> {
        generate_price_feed(vec![
            ("USD", "1.00", 0),
            ("EURO", "1.0526", 0),
            ("GDP", "1.075", 0),
            ("JPY", "0.0094", 0),
        ])
    }

    #[test]
    fn test_freeze_and_rollback() {
        let prices: Vec<(String, Uint128)> = feed_2()
            .iter()
            .map(|p| (p.key.clone(), p.data.rate.try_into().unwrap()))
            .collect();
        let new_prices: Vec<(String, Uint128)> = feed_3()
            .iter()
            .map(|p| (p.key.clone(), p.data.rate.try_into().unwrap()))
            .collect();
        let TestScenario {
            mut app,
            router,
            admin,
            provider,
            user,
            ..
        } = TestScenario::new(prices);
        let target = Uint256::from_u128(105 * 10u128.pow(16));
        let symbol = "SILK".to_string();
        let basket = basic_basket();
        let t2 = SIX_HOURS + 10;
        let index_oracle = IndexOracleHelper::init(
            &admin,
            &mut app,
            &router.clone().into(),
            &basket,
            target,
            &symbol,
            SIX_HOURS,
            Decimal256::percent(10),
        );

        // Configure router w/ index oracle
        router
            .set_keys(
                &admin,
                &mut app,
                index_oracle.0.clone().into(),
                vec![symbol.clone()],
            )
            .unwrap();

        let price = router.query_price(&app, symbol.clone()).unwrap();
        MathAsserter::within_deviation(target, price.data.rate, TestScenario::ERROR);

        app.update_block(|b| b.time = b.time.plus_seconds(t2));

        // Prices have become stale, peg is frozen
        index_oracle.compute_index(&admin, &mut app).unwrap();
        let price = router.query_price(&app, symbol.clone()).unwrap();
        MathAsserter::within_deviation(target, price.data.rate, TestScenario::ERROR);
        let info = index_oracle.query_index_data(&app).unwrap();
        assert!(info.peg.frozen);
        assert_eq!(info.peg.last_updated, Uint64::zero());
        assert!(index_oracle.unfreeze(&admin, &mut app).is_err());

        // Push new prices
        let (_, new_prices) = OracleCore::create_prices_hashmap(new_prices);
        provider.update_band_prices(&admin, &mut app, new_prices, Some(t2));

        // Trigger unfreeze, which will perform a rollback
        assert!(index_oracle.unfreeze(&user, &mut app).is_err());
        assert!(index_oracle.unfreeze(&admin, &mut app).is_ok());

        let price = router.query_price(&app, symbol.clone()).unwrap();
        MathAsserter::within_deviation(target, price.data.rate, TestScenario::ERROR);
        let info = index_oracle.query_index_data(&app).unwrap();
        assert!(!info.peg.frozen);
        assert_eq!(info.peg.last_updated, Uint64::new(t2));
    }

    #[test]
    fn test_status_and_permissions() {
        let prices: Vec<(String, Uint128)> = feed_0()
            .iter()
            .map(|p| (p.key.clone(), p.data.rate.try_into().unwrap()))
            .collect();
        let TestScenario {
            mut app,
            router,
            admin,
            user,
            ..
        } = TestScenario::new(prices);
        let target = Uint256::from_u128(105 * 10u128.pow(16));
        let symbol = "SILK".to_string();
        let basket = basic_basket();

        let index_oracle = IndexOracleHelper::init(
            &admin,
            &mut app,
            &router.clone().into(),
            &basket,
            target,
            &symbol,
            SIX_HOURS,
            Decimal256::percent(10),
        );

        router
            .set_keys(
                &admin,
                &mut app,
                index_oracle.0.clone().into(),
                vec![symbol.clone()],
            )
            .unwrap();

        let original_config = index_oracle.query_index_data(&app).unwrap();

        // Can only update status when frozen

        assert!(index_oracle
            .update_status(&user, &mut app, ContractStatus::Frozen)
            .is_err());
        assert!(index_oracle
            .update_status(&admin, &mut app, ContractStatus::Frozen)
            .is_ok());

        assert!(index_oracle.compute_index(&user, &mut app).is_err());
        assert!(index_oracle.compute_index(&admin, &mut app).is_err());
        assert!(index_oracle.unfreeze(&admin, &mut app).is_err());

        assert!(router.query_price(&app, symbol.clone()).is_err());
        assert!(router.query_prices(&app, vec![symbol.clone()]).is_err());

        // Can't query prices when deprecated

        assert!(index_oracle
            .update_status(&admin, &mut app, ContractStatus::Deprecated)
            .is_ok());
        assert!(router.query_price(&app, symbol.clone()).is_err());
        assert!(router.query_prices(&app, vec![symbol.clone()]).is_err());

        // Config update works and is admin only
        let new_when_stale = SIX_HOURS + 1u64;
        assert!(index_oracle
            .update_status(&admin, &mut app, ContractStatus::Normal)
            .is_ok());
        assert!(index_oracle
            .update_config(&user, &mut app, None, None, Some(new_when_stale), None)
            .is_err());
        assert!(index_oracle
            .update_config(&admin, &mut app, None, None, Some(new_when_stale), None)
            .is_ok());

        let new_config = index_oracle.query_index_data(&app).unwrap();
        assert_ne!(original_config.when_stale, new_config.when_stale);
        assert_eq!(new_config.when_stale.u64(), new_when_stale);
    }

    #[rstest]
    #[case(
            "sUSD",
            vec![
                ("USD", "1.00"), // 100%
            ],
            vec![
                ("USD", 10u128.pow(18)), // $1
            ],
            10u128.pow(18).into(), // $1
            10u128.pow(18).into(), // $1
            (1_05 * 10u128.pow(16)).into(),
            (1_05 * 10u128.pow(16)).into(),
    )]
    #[case(
        "INDEX",
        vec![
            ("USD", "0.25"), // 25%
            ("BTC", "0.25"), // 25%
            ("ETH", "0.25"), // 25%
            ("XAU", "0.25"), // 25%
        ],
        vec![
            ("USD", 1_00 * 10u128.pow(14)), // $1
            ("BTC", 29_398_20 * 10u128.pow(14)), // $29398.2
            ("ETH", 1_831_26 * 10u128.pow(14)), // $1831.26
            ("XAU", 1_852_65 * 10u128.pow(14)), // $1852.65
        ],
        (10u128.pow(18)).into(),
        (10u128.pow(18)).into(),
        (1_15 * 10u128.pow(16)).into(),
        (1_15 * 10u128.pow(16)).into(),
    )]
    #[case(
            "SILK",
            vec![
                ("USD", "0.3933"), //  39.33%
                ("CNY", "0.0713"), //  7.13%
                ("EUR", "0.1597"), // 15.97%
                ("JPY", "0.0764"), //  7.64%
                ("GBP", "0.0340"), //  3.4%
                ("CAD", "0.0458"), //  4.58%
                ("KRW", "0.0153"), //  1.53%
                ("AUD", "0.0232"), //  2.32%
                ("IDR", "0.0250"), //  2.5%
                ("CHF", "0.0444"), //  4.44%
                ("SEK", "0.0084"), //  0.84%
                ("NOK", "0.0082"), //  0.82%
                ("SGD", "0.025"), //  2.5%
                ("XAU", "0.05"), //  5.0%
                ("BTC", "0.02"), //  2.0%
            ],
            vec![
                ("USD", 1_00 * 10u128.pow(14)), // $1
                ("CNY", 0_15 * 10u128.pow(14)), // $0.15
                ("EUR", 1_07 * 10u128.pow(14)), // $1.07
                ("JPY", 0_0079 * 10u128.pow(12)), // $0.0079
                ("GBP", 1_26 * 10u128.pow(14)), // $1.26
                ("CAD", 0_78 * 10u128.pow(14)), // $0.78
                ("KRW", 0_00079 * 10u128.pow(11)), // $0.00079
                ("AUD", 0_71 * 10u128.pow(14)), // $0.71
                ("IDR", 0_000068 * 10u128.pow(10)), // $0.000068
                ("CHF", 1_04 * 10u128.pow(14)), // $1.04
                ("SEK", 0_10 * 10u128.pow(14)), // $0.10
                ("NOK", 0_10 * 10u128.pow(14)), // $0.10
                ("SGD", 0_73 * 10u128.pow(14)), // $0.73
                ("XAU", 1_852_65 * 10u128.pow(14)), // $1852.65
                ("BTC", 29_398_20 * 10u128.pow(14)), // $29398.2
            ],
            (1_05 * 10u128.pow(16)).into(),
            (1_05 * 10u128.pow(16)).into(),
            (10u128.pow(18)).into(),
            (10u128.pow(18)).into(),
        )]
    fn test_index_math(
        #[case] symbol: String,
        #[case] basket: Vec<(&str, &str)>,
        #[case] prices: Vec<(&str, u128)>,
        #[case] target: Uint256,
        #[case] expected: Uint256,
        #[case] new_target: Uint256,
        #[case] expected_new: Uint256,
    ) {
        let basket = IndexOracleHelper::create_basket(basket);
        let TestScenario {
            mut app,
            router,
            admin,
            user,
            ..
        } = TestScenario::new(prices);

        let index_oracle = IndexOracleHelper::init(
            &admin,
            &mut app,
            &router.clone().into(),
            &basket,
            target,
            &symbol,
            SIX_HOURS,
            Decimal256::percent(10),
        );

        // Configure router w/ index oracle
        router
            .set_keys(
                &admin,
                &mut app,
                index_oracle.0.clone().into(),
                vec![symbol.clone()],
            )
            .unwrap();

        let price = router.query_price(&app, symbol.clone()).unwrap();
        let prices = router.query_prices(&app, vec![symbol.clone()]).unwrap();
        assert_eq!(price, prices[0]);
        let data = price.data();

        MathAsserter::within_deviation(expected, data.rate, TestScenario::ERROR);

        assert!(index_oracle
            .update_target(&user, &mut app, new_target)
            .is_err());
        assert!(index_oracle
            .update_target(&admin, &mut app, new_target)
            .is_ok());

        let price = router.query_price(&app, symbol.clone()).unwrap();
        let prices = router.query_prices(&app, vec![symbol]).unwrap();
        assert_eq!(price, prices[0]);
        let data = price.data();
        MathAsserter::within_deviation(expected_new, data.rate, TestScenario::ERROR);
    }

    /* - Setup oracle with symbol, basket, prices, & target -- check against expected_initial
     * - Change to new_prices & apply mod_basket changes -- check against expected_final
     */
    #[rstest]
    #[case(
            "sUSD",
            // basket
            vec![
                ("USD", "1.00"), // 100%
            ],
            // prices
            vec![
                ("USD", 10u128.pow(18)), // $1
                ("BTC", 30_000 * 10u128.pow(18)), // $30,000
            ],
            // new prices
            vec![
                ("USD", 98 * 10u128.pow(16)), // $0.98
            ],
            // mod basket
            vec![
                ("USD", "0"), // 0% (remove)
                ("BTC", "1.00"), // 100%
            ],
            //expected basket
            vec![
                ("BTC", "1.00"), // 100%
            ],
            // target
            10u128.pow(18), // $1
            // expected initial
            10u128.pow(18), // $1
            // expected final
            98 * 10u128.pow(16), // $0.98
        )]
    #[case(
            "AnIndex",
            // basket
            vec![
                ("USD", "0.25"), // 25%
                ("BTC", "0.30"), // 30%
                ("ETH", "0.45"), // 45%
            ],
            // prices
            vec![
                ("USD", 10u128.pow(18)), // $1
                ("ATOM", 30 * 10u128.pow(18)), // $30
                ("BTC", 30_000 * 10u128.pow(18)), // $30,000
                ("ETH", 2_000 * 10u128.pow(18)), // $2,000
            ],
            // new prices
            vec![
                ("USD", 0_03 * 10u128.pow(16)), // $0.03
                ("BTC", 45_000 * 10u128.pow(18)), // $45,000
                ("ETH", 3_000 * 10u128.pow(18)), // $3,000
            ],
            // mod basket
            vec![
                ("USD", "0"), // 0% (remove)
                ("BTC", "0.10"), // decrease to 10%
                ("ATOM", "0.45"), // add at 45%
            ],
            //expected basket
            vec![
                ("BTC", "0.10"), // 10%
                ("ATOM", "0.45"), // 45%
                ("ETH", "0.45"), // 45%
            ],
            // target
            10 * 10u128.pow(18), // $10
            // expected initial
            10 * 10u128.pow(18), // $10
            // expected final
            11_325 * 10u128.pow(15), // $11.325
        )
    ]
    fn test_mod_basket(
        #[case] symbol: String,
        #[case] basket: Vec<(&str, &str)>,
        #[case] prices: Vec<(&str, u128)>,
        #[case] new_prices: Vec<(&str, u128)>,
        #[case] mod_basket: Vec<(&str, &str)>,
        #[case] expected_weights: Vec<(&str, &str)>,
        #[case] target: u128,
        #[case] expected_initial: u128,
        #[case] expected_final: u128,
    ) {
        let basket = IndexOracleHelper::create_basket(basket);
        let mod_basket: Vec<(String, Decimal256)> = IndexOracleHelper::create_basket(mod_basket);
        let expected_weights: Vec<(String, Decimal256)> = expected_weights
            .into_iter()
            .map(|(sym, w)| (sym.to_string(), Decimal256::from_str(w).unwrap()))
            .collect();
        let target: Uint256 = target.into();
        let expected_initial: Uint256 = expected_initial.into();
        let expected_final: Uint256 = expected_final.into();

        let new_prices: HashMap<String, Uint128> = OracleCore::create_prices_hashmap(new_prices).1;

        let TestScenario {
            mut app,
            router,
            admin,
            user,
            provider,
            ..
        } = TestScenario::new(prices);

        let index_oracle = IndexOracleHelper::init(
            &admin,
            &mut app,
            &router.clone().into(),
            &basket,
            target,
            &symbol,
            SIX_HOURS,
            Decimal256::percent(20),
        );

        // Configure router w/ index oracle
        router
            .set_keys(
                &admin,
                &mut app,
                index_oracle.0.clone().into(),
                vec![symbol.clone()],
            )
            .unwrap();

        let price = router.query_price(&app, symbol.clone()).unwrap();

        let data = price.data();

        MathAsserter::within_deviation(expected_initial, data.rate, TestScenario::ERROR);

        // Update mock provider prices
        provider.update_band_prices(&admin, &mut app, new_prices, None);

        let price = router.query_price(&app, symbol.clone()).unwrap();

        let data = price.data();

        MathAsserter::within_deviation(expected_final, data.rate, TestScenario::ERROR);

        // Update basket
        assert!(index_oracle
            .mod_basket(&user, &mut app, &mod_basket)
            .is_err());
        assert!(index_oracle
            .mod_basket(&admin, &mut app, &mod_basket)
            .is_ok());

        // check basket changed
        let BasketResponse { mut basket } = index_oracle.query_basket(&app).unwrap();
        {
            basket.sort();
            for (sym, w, _) in basket {
                assert!(
                    expected_weights.contains(&(sym.clone(), w)),
                    "Bad symbol found {}, {}",
                    sym,
                    w
                );
            }
        };

        // check price doesn't change on mod_price
        let price = router.query_price(&app, symbol.clone()).unwrap();

        MathAsserter::within_deviation(expected_final, price.data().rate, TestScenario::ERROR);
    }

    #[test]
    fn test_peg_deviation() {
        let prices: Vec<(String, Uint128)> = feed_2()
            .iter()
            .map(|p| (p.key.clone(), p.data.rate.try_into().unwrap()))
            .collect();
        let TestScenario {
            mut app,
            router,
            admin,
            provider,
            ..
        } = TestScenario::new(prices);
        let target = Uint256::from_u128(1 * 10u128.pow(18));
        let symbol = "SILK".to_string();
        let world_basket = basic_basket();
        let some_symbol = world_basket[0].0.clone();
        let index_oracle = IndexOracleHelper::init(
            &admin,
            &mut app,
            &router.clone().into(),
            &world_basket,
            target,
            &symbol,
            SIX_HOURS,
            Decimal256::percent(10),
        );

        // Configure router w/ index oracle
        router
            .set_keys(
                &admin,
                &mut app,
                index_oracle.0.clone().into(),
                vec![symbol.clone()],
            )
            .unwrap();

        let price = router.query_price(&app, symbol.clone()).unwrap();
        MathAsserter::within_deviation(target, price.data.rate, TestScenario::ERROR);

        let new_prices = OracleCore::create_prices_hashmap(vec![(
            some_symbol.clone(),
            1_000_000 * 10u128.pow(18),
        )])
        .1;
        provider.update_band_prices(&admin, &mut app, new_prices, None);
        assert!(router.query_price(&app, symbol.clone()).is_err());
    }

    #[test]
    fn test_multiple_basket_updates() {
        let prices: Vec<(String, Uint128)> = feed_2()
            .iter()
            .map(|p| (p.key.clone(), p.data.rate.try_into().unwrap()))
            .collect();
        let TestScenario {
            mut app,
            router,
            admin,
            ..
        } = TestScenario::new(prices);
        let target = Uint256::from_u128(1 * 10u128.pow(18));
        let symbol = "SILK".to_string();
        let usd_basket = usd_basket();
        let world_basket = basic_basket();
        let index_oracle = IndexOracleHelper::init(
            &admin,
            &mut app,
            &router.clone().into(),
            &usd_basket,
            target,
            &symbol,
            SIX_HOURS,
            Decimal256::percent(10),
        );

        // Configure router w/ index oracle
        router
            .set_keys(
                &admin,
                &mut app,
                index_oracle.0.clone().into(),
                vec![symbol.clone()],
            )
            .unwrap();

        let price = router.query_price(&app, symbol.clone()).unwrap();
        MathAsserter::within_deviation(target, price.data.rate, TestScenario::ERROR);

        let basket_states = vec![&world_basket, &usd_basket, &world_basket];
        for basket in basket_states {
            assert!(index_oracle.mod_basket(&admin, &mut app, &basket).is_ok());
            let price = router.query_price(&app, symbol.clone()).unwrap();
            MathAsserter::within_deviation(target, price.data.rate, TestScenario::ERROR);
        }
    }
}
