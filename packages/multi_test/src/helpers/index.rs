use std::str::FromStr;

use super::*;
use crate::harness::index::IndexOracle;
use shade_oracles::{core::Query, interfaces::index::msg::*};

create_test_helper!(IndexOracleHelper);
impl IndexOracleHelper {
    pub fn init(
        user: &User,
        app: &mut App,
        router: &Contract,
        basket: &[(String, Decimal256)],
        target: Uint128,
        symbol: &str,
        when_stale: u64,
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
                },
                IndexOracle::default(),
                "index_oracle",
            )
            .unwrap();
        Self(contract)
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

    pub fn query_basket(&self, app: &App) -> StdResult<BasketResponse> {
        QueryMsg::GetBasket {}.test_query(&self.0, app)
    }

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
    use shade_oracles::interfaces::index::SIX_HOURS;
    use std::str::FromStr;

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
            10u128.pow(10).into(), // .000001% error
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
        (10u128.pow(10)).into(), // .000001% error
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
            10u128.pow(10).into() // .000001% error
        )]
    fn basic_index_test(
        #[case] symbol: String,
        #[case] basket: Vec<(&str, &str)>,
        #[case] prices: Vec<(&str, u128)>,
        #[case] target: Uint128,
        #[case] expected: Uint256,
        #[case] error: Uint128,
    ) {
        let basket = IndexOracleHelper::create_basket(basket);
        let TestScenario {
            mut app,
            router,
            admin,
            ..
        } = TestScenario::new(prices);
        let user = admin;

        let index_oracle = IndexOracleHelper::init(
            &user,
            &mut app,
            &router.clone().into(),
            &basket,
            target,
            &symbol,
            SIX_HOURS,
        );

        // Configure router w/ index oracle
        router
            .set_keys(&user, &mut app, index_oracle.0.into(), vec![symbol.clone()])
            .unwrap();

        let price = router.query_price(&app, symbol).unwrap();
        let data = price.data();

        {
            let err = if data.rate > expected {
                data.rate - expected
            } else {
                expected - data.rate
            };
            let acceptable = expected.multiply_ratio(error, 10u128.pow(18));

            assert!(
                err <= acceptable,
                "price: {}, expected: {}, exceeds acceptable error",
                data.rate,
                expected
            );
        };
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
            10u128.pow(10), // .000001% error
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
            10u128.pow(10), // .000001% error
        )
    ]
    fn mod_index_test(
        #[case] symbol: String,
        #[case] basket: Vec<(&str, &str)>,
        #[case] prices: Vec<(&str, u128)>,
        #[case] new_prices: Vec<(&str, u128)>,
        #[case] mod_basket: Vec<(&str, &str)>,
        #[case] expected_weights: Vec<(&str, &str)>,
        #[case] target: u128,
        #[case] expected_initial: u128,
        #[case] expected_final: u128,
        #[case] error: u128,
    ) {
        let basket = IndexOracleHelper::create_basket(basket);
        let mod_basket: Vec<(String, Decimal256)> = IndexOracleHelper::create_basket(mod_basket);
        let expected_weights: Vec<(String, Decimal256)> = expected_weights
            .into_iter()
            .map(|(sym, w)| (sym.to_string(), Decimal256::from_str(w).unwrap()))
            .collect();
        let target: Uint128 = target.into();
        let expected_initial: Uint128 = expected_initial.into();
        let expected_final: Uint128 = expected_final.into();
        let error: Uint128 = error.into();

        let new_prices: HashMap<String, Uint128> = OracleCore::create_prices_hashmap(new_prices).1;

        let TestScenario {
            mut app,
            router,
            admin,
            band,
            ..
        } = TestScenario::new(prices);
        let user = admin;

        let index_oracle = IndexOracleHelper::init(
            &user,
            &mut app,
            &router.clone().into(),
            &basket,
            target,
            &symbol,
            SIX_HOURS,
        );

        // Configure router w/ index oracle
        router
            .set_keys(
                &user,
                &mut app,
                index_oracle.0.clone().into(),
                vec![symbol.clone()],
            )
            .unwrap();

        let price = router.query_price(&app, symbol.clone()).unwrap();

        let data = price.data();
        let expected_initial = Uint256::from_uint128(expected_initial);
        {
            let err = if data.rate > expected_initial {
                data.rate - expected_initial
            } else {
                expected_initial - data.rate
            };
            let acceptable = expected_initial.multiply_ratio(error, 10u128.pow(18));

            assert!(
                err <= acceptable,
                "price: {}, expected: {}, exceeds acceptable error",
                data.rate,
                expected_initial
            );
        };

        // Update mock band prices
        band.update_prices(&user, &mut app, new_prices, None);

        let price = router.query_price(&app, symbol.clone()).unwrap();

        let data = price.data();
        let expected_final = Uint256::from_uint128(expected_final);

        {
            let err = if data.rate > expected_final {
                data.rate - expected_final
            } else {
                expected_final - data.rate
            };
            let acceptable = expected_final.multiply_ratio(error, 10u128.pow(18));

            assert!(
                err <= acceptable,
                "Price change check failed price: {}, expected: {}, exceeds acceptable error",
                data.rate,
                expected_final
            );
        };

        // Update basket
        index_oracle
            .mod_basket(&user, &mut app, &mod_basket)
            .unwrap();

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
        let data = price.data();
        {
            let err = if data.rate > expected_final {
                data.rate - expected_final
            } else {
                expected_final - data.rate
            };
            let acceptable = expected_final.multiply_ratio(error, 10u128.pow(18));

            assert!(
                err <= acceptable,
                "Post-Mod price: {}, expected: {}, exceeds acceptable error",
                data.rate,
                expected_final
            );
        };
    }
}
