use super::*;
use crate::harness::derivatives::BotStakingDerivativesOracle;
use shade_oracles::interfaces::{common::config::CommonConfigResponse, derivatives::bot::*};

create_test_helper!(BotStakingDerivativesOracleHelper);

impl BotStakingDerivativesOracleHelper {
    pub fn init(sender: &User, app: &mut App, router: &Contract) -> Self {
        let contract = sender
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                },
                BotStakingDerivativesOracle::default(),
                "bot_staking_derivatives_oracle",
            )
            .unwrap();
        Self(contract)
    }

    pub fn set_derivatives(
        &self,
        sender: &User,
        app: &mut App,
        derivatives: &[RawDerivativeData],
    ) -> AnyResult<AppResponse> {
        sender.exec(
            app,
            &ExecuteMsg::SetDerivatives(derivatives.to_vec()),
            &self.0,
        )
    }

    pub fn remove_derivatives(
        &self,
        sender: &User,
        app: &mut App,
        keys: &[String],
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::RemoveDerivatives(keys.to_vec()), &self.0)
    }

    pub fn update_derivatives(
        &self,
        sender: &User,
        app: &mut App,
        update: DerivativeUpdates,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::UpdateDerivatives(update), &self.0)
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

    pub fn query_derivatives(&self, app: &App) -> StdResult<DerivativesResponse> {
        QueryMsg::GetDerivatives {}.test_query(&self.0, app)
    }
    pub fn query_config(&self, app: &App) -> StdResult<CommonConfigResponse> {
        QueryMsg::GetConfig {}.test_query(&self.0, app)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use cosmwasm_std::{Addr, StdError};
    use multi_test_helpers::Asserter;
    use shade_oracles::{
        error::CommonOracleError,
        interfaces::{
            common::{config::BotPermission, OraclePrice},
            providers::ReferenceData,
        },
        unit_test_interface::prices::PricesFixture,
    };

    fn derivative_data() -> Vec<RawDerivativeData> {
        vec![
            create_derivative_data("stkd-ETH", "ETH", "1.1", 1, "0.1"),
            create_derivative_data("stkd-OSMO", "OSMO", "1.2", 1, "0.1"),
            create_derivative_data("stkd-FRAX", "FRAX", "1.5", 1, "0.1"),
        ]
    }

    fn create_derivative_data(
        key: &'static str,
        underlying_key: &'static str,
        rate: &'static str,
        rate_timeout: u64,
        rate_max_change: &'static str,
    ) -> RawDerivativeData {
        RawDerivativeData {
            key: key.to_string(),
            underlying_key: underlying_key.to_string(),
            initial_rate: Decimal256::from_str(rate).unwrap(),
            rate_timeout,
            rate_max_change: Decimal256::from_str(rate_max_change).unwrap(),
        }
    }

    #[test]
    fn test_bot_rates_update() {
        let prices = PricesFixture::basic_prices_2();
        let TestScenario {
            mut app,
            admin,
            user,
            router,
            admin_auth,
            ..
        } = TestScenario::new(prices);
        let app = &mut app;
        let oracle = BotStakingDerivativesOracleHelper::init(&user, app, &router.into());
        let derivatives = derivative_data();

        let rate_bot = User::new("bot");
        admin_auth.register_admin(&admin, app, rate_bot.str());

        assert!(oracle.set_derivatives(&admin, app, &derivatives).is_ok());

        let update_rates_too_much_upside = DerivativeUpdates::Rates(vec![(
            "stkd-ETH".to_string(),
            Decimal256::from_str("1.22").unwrap(),
        )]);

        let update_rates_too_much_downside = DerivativeUpdates::Rates(vec![(
            "stkd-ETH".to_string(),
            Decimal256::from_str("0.98").unwrap(),
        )]);

        let okay_rates = vec![
            (
                "stkd-ETH".to_string(),
                Decimal256::from_str("1.15").unwrap(),
            ),
            (
                "stkd-OSMO".to_string(),
                Decimal256::from_str("1.2").unwrap(),
            ),
            (
                "stkd-FRAX".to_string(),
                Decimal256::from_str("1.48").unwrap(),
            ),
        ];
        let okay_rate_update = DerivativeUpdates::Rates(okay_rates.clone());

        assert!(oracle
            .update_derivatives(&rate_bot, app, okay_rate_update.clone())
            .is_err());
        admin_auth.grant_access(
            &admin,
            app,
            rate_bot.str(),
            vec![BotPermission::UpdateStrideRates.to_string()],
        );
        // Update > max upside
        assert!(oracle
            .update_derivatives(&rate_bot, app, update_rates_too_much_upside)
            .is_err());
        assert!(oracle
            .update_derivatives(&rate_bot, app, update_rates_too_much_downside)
            .is_err());
        assert!(oracle
            .update_derivatives(&rate_bot, app, okay_rate_update)
            .is_ok());

        let derivatives = oracle.query_derivatives(app).unwrap();
        let actual_rates = derivatives.iter().map(|d| d.rate.value).collect::<Vec<_>>();
        let expected_rates = okay_rates.iter().map(|(_, r)| *r).collect::<Vec<_>>();
        Asserter::equal_vecs(&actual_rates, &expected_rates);
    }

    #[test]
    fn test_registry() {
        let prices = PricesFixture::basic_prices_2();
        let TestScenario {
            mut app,
            admin,
            user,
            router,
            ..
        } = TestScenario::new(prices);
        let app = &mut app;
        let oracle = BotStakingDerivativesOracleHelper::init(&user, app, &router.into());
        let raw_derivatives = derivative_data();

        assert!(oracle
            .set_derivatives(&user, app, &raw_derivatives)
            .is_err());
        assert!(oracle
            .set_derivatives(&admin, app, &raw_derivatives)
            .is_ok());

        let derivatives = oracle.query_derivatives(app).unwrap();
        let config = oracle.query_config(app).unwrap();
        assert_eq!(derivatives.len(), raw_derivatives.len());
        let mut expected_keys = vec![];
        for d in raw_derivatives.clone() {
            expected_keys.push(d.key.clone());
            expected_keys.push(format!("{}{}", d.key, " Rate"));
        }
        Asserter::equal_vecs(&config.supported_keys, &expected_keys);

        assert!(oracle
            .remove_derivatives(&user, app, &[raw_derivatives[1].key.clone()])
            .is_err());
        assert!(oracle
            .remove_derivatives(&admin, app, &[raw_derivatives[1].key.clone()])
            .is_ok());
        let derivatives = oracle.query_derivatives(app).unwrap();
        assert_eq!(derivatives.len(), raw_derivatives.len() - 1);
        assert!(!derivatives.iter().any(|d| d.key == raw_derivatives[1].key));

        let config = oracle.query_config(app).unwrap();
        let mut expected_keys = vec![];
        for d in raw_derivatives.clone() {
            if d.key == raw_derivatives[1].key {
                continue;
            }
            expected_keys.push(d.key.clone());
            expected_keys.push(format!("{}{}", d.key, " Rate"));
        }
        Asserter::equal_vecs(&config.supported_keys, &expected_keys);

        let now = app.block_info().time.seconds();
        let new_derivative = DerivativeData::new(
            raw_derivatives[0].key.clone(),
            PricesFixture::XAU.to_string(),
            raw_derivatives[0].initial_rate,
            Decimal256::from_str("0.1").unwrap(),
            2000u64,
            now,
        )
        .unwrap();
        let valid_update = DerivativeUpdates::Config(vec![(
            raw_derivatives[0].key.clone(),
            DerivativeDataConfigUpdate::new(
                Some(PricesFixture::XAU.to_string()),
                Some(new_derivative.rate.timeout),
                Some(new_derivative.rate.max_change),
            ),
        )]);
        oracle
            .update_derivatives(&admin, app, valid_update)
            .unwrap();
        let derivatives = oracle.query_derivatives(app).unwrap();
        let derivative = derivatives
            .iter()
            .find(|d| d.key == raw_derivatives[0].key)
            .unwrap();
        assert_eq!(derivative, &new_derivative);
    }

    #[test]
    fn test_common_config() {
        let prices = PricesFixture::basic_prices_2();
        let TestScenario {
            mut app,
            admin,
            user,
            router,
            ..
        } = TestScenario::new(prices);
        let app = &mut app;
        let oracle = BotStakingDerivativesOracleHelper::init(&user, app, &router.into());
        assert!(oracle.set_status(&user, app, false).is_err());
        assert!(oracle.set_status(&admin, app, false).is_ok());
        let new_router = RawContract {
            address: "new_router".to_string(),
            code_hash: "new_router".to_string(),
        };
        assert!(oracle.update_config(&admin, app, &new_router).is_err());
        oracle.set_status(&admin, app, true).unwrap();
        assert!(oracle.update_config(&admin, app, &new_router).is_ok());

        let config = oracle.query_config(app).unwrap();
        assert_eq!(
            config.config.router,
            Contract {
                address: Addr::unchecked("new_router"),
                code_hash: "new_router".to_string()
            }
        );
    }

    #[test]
    fn test_price_calculation() {
        let prices = PricesFixture::basic_prices_2();
        let TestScenario {
            mut app,
            admin,
            user,
            router,
            ..
        } = TestScenario::new(prices);
        let app = &mut app;
        let oracle = BotStakingDerivativesOracleHelper::init(&user, app, &router.clone().into());
        let derivatives = derivative_data();
        assert!(oracle.set_derivatives(&admin, app, &derivatives).is_ok());
        router
            .set_keys(
                &admin,
                app,
                oracle.0.clone().into(),
                vec![
                    "stkd-ETH".to_string(),
                    "stkd-OSMO".to_string(),
                    "stkd-OSMO Rate".to_string(),
                ],
            )
            .unwrap();
        let eth_price = router.query_price(app, PricesFixture::ETH.into()).unwrap();
        let osmo_price = router.query_price(app, PricesFixture::OSMO.into()).unwrap();
        let expected_price = eth_price.data.rate * derivatives[0].initial_rate;
        let expected_osmo_price = osmo_price.data.rate * derivatives[1].initial_rate;
        let actual_price = router
            .query_price(app, "stkd-ETH".into())
            .unwrap()
            .data
            .rate;
        assert_eq!(expected_price, actual_price);
        let actual_price = router
            .query_prices(
                app,
                vec![
                    "stkd-ETH".into(),
                    "stkd-OSMO".into(),
                    "stkd-OSMO Rate".into(),
                ],
            )
            .unwrap();
        assert_eq!(
            vec![
                expected_price,
                expected_osmo_price,
                derivatives[1].initial_rate.atomics()
            ],
            vec![
                actual_price[0].data.rate,
                actual_price[1].data.rate,
                actual_price[2].data.rate
            ]
        );
        app.update_block(|b| b.time = b.time.plus_seconds(3u64));
        assert!(router.query_price(app, "stkd-ETH".into()).is_err());
    }
}
