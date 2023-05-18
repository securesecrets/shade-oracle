use super::*;
use crate::harness::derivatives::ShadeStakingDerivativesOracle;
use shade_oracles::interfaces::{common::config::CommonConfigResponse, derivatives::generic::*};

create_test_helper!(GenericStakingDerivativesOracleHelper);

impl GenericStakingDerivativesOracleHelper {
    pub fn init_shade_v1(sender: &User, app: &mut App, router: &Contract) -> Self {
        let contract = sender
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                },
                ShadeStakingDerivativesOracle::default(),
                "shade_staking_derivatives_oracle",
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
    use crate::mocks::MockShadeStkdScrtHelper;

    use super::*;
    use cosmwasm_std::Addr;
    use multi_test_helpers::Asserter;
    use shade_oracles::{
        interfaces::{common::OraclePrice, providers::ReferenceData},
        unit_test_interface::prices::PricesFixture,
    };

    #[allow(clippy::type_complexity)]
    fn derivatives_fixture(
        user: &User,
        app: &mut App,
    ) -> (
        Vec<String>,
        Vec<String>,
        Vec<Uint128>,
        Vec<MockShadeStkdScrtHelper>,
        Vec<RawDerivativeData>,
    ) {
        let values = vec![
            ("stkd-SCRT", PricesFixture::SCRT, 6, 1_1 * 10u128.pow(5)),
            ("stkd-SHD", PricesFixture::SHD, 12, 1_1 * 10u128.pow(11)),
            ("stkd-ETH", PricesFixture::ETH, 18, 1_2 * 10u128.pow(17)),
        ];
        let mut helpers = vec![];
        let mut data = vec![];
        let mut keys = vec![];
        let mut underlying_keys = vec![];
        let mut rates = vec![];
        for (key, underlying_key, decimals, rate) in values {
            let d = create_derivative(user, app, key, key, decimals, Uint128::new(rate));
            helpers.push(d.clone());
            data.push(d.to_raw_derivative_data(key, underlying_key));
            keys.push(key.to_string());
            rates.push(Uint128::new(rate));
            underlying_keys.push(underlying_key.to_string());
        }
        (keys, underlying_keys, rates, helpers, data)
    }

    fn create_derivative(
        user: &User,
        app: &mut App,
        name: &str,
        symbol: &str,
        decimals: u8,
        price: Uint128,
    ) -> MockShadeStkdScrtHelper {
        MockShadeStkdScrtHelper::init(
            user,
            app,
            name.to_string(),
            symbol.to_string(),
            decimals,
            price,
        )
    }

    #[test]
    fn test_registry() {
        let TestScenario {
            mut app,
            admin,
            user,
            router,
            ..
        } = TestScenario::new(PricesFixture::basic_prices_2());
        let app = &mut app;
        let oracle =
            GenericStakingDerivativesOracleHelper::init_shade_v1(&user, app, &router.into());
        let (.., data) = derivatives_fixture(&user, app);

        let set_data = data[0..=1].to_vec();

        assert!(oracle.set_derivatives(&user, app, &set_data).is_err());
        assert!(oracle.set_derivatives(&admin, app, &set_data).is_ok());

        let derivatives = oracle.query_derivatives(app).unwrap();
        assert_eq!(derivatives.len(), 2);
        let config = oracle.query_config(app).unwrap();
        assert_eq!(config.supported_keys.len(), 4);

        let actual = derivatives
            .iter()
            .map(|d| d.staking_derivative.contract.address.to_string())
            .collect::<Vec<_>>();
        let expected = set_data
            .iter()
            .map(|d| d.staking_derivative.contract.address.to_string())
            .collect::<Vec<_>>();
        Asserter::equal_vecs(&actual, &expected);

        oracle
            .remove_derivatives(&admin, app, &[set_data[0].key.clone()])
            .unwrap();

        let derivatives = oracle.query_derivatives(app).unwrap();
        assert_eq!(derivatives.len(), 1);
        let config = oracle.query_config(app).unwrap();
        assert_eq!(config.supported_keys.len(), 2);
        assert!(!derivatives.iter().any(|d| { d.key == set_data[0].key }));

        let new_data = RawDerivativeData {
            key: derivatives[0].key.clone(),
            staking_derivative: data[2].staking_derivative.clone(),
        };
        assert!(oracle.set_derivatives(&admin, app, &[new_data]).is_ok());

        let derivatives = oracle.query_derivatives(app).unwrap();
        assert_eq!(derivatives.len(), 1);
        assert!(derivatives[0]
            .staking_derivative
            .contract
            .address
            .eq(&data[2].staking_derivative.contract.address));
    }

    #[test]
    fn test_common_config() {
        let TestScenario {
            mut app,
            admin,
            user,
            router,
            ..
        } = TestScenario::new(PricesFixture::basic_prices_2());
        let app = &mut app;
        let oracle =
            GenericStakingDerivativesOracleHelper::init_shade_v1(&user, app, &router.into());
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
    fn test_price_calculations() {
        let TestScenario {
            mut app,
            admin,
            user,
            router,
            ..
        } = TestScenario::new(PricesFixture::basic_prices_2());
        let app = &mut app;
        let oracle = GenericStakingDerivativesOracleHelper::init_shade_v1(
            &user,
            app,
            &router.clone().into(),
        );
        let (keys, underlying_keys, .., data) = derivatives_fixture(&user, app);

        assert!(oracle.set_derivatives(&admin, app, &data).is_ok());

        let derivative_rate_keys = keys.iter().map(|d| d.clone() + " Rate").collect::<Vec<_>>();
        let all_keys = keys
            .iter()
            .cloned()
            .chain(derivative_rate_keys.iter().cloned())
            .collect();
        router
            .set_keys(&admin, app, oracle.0.clone().into(), all_keys)
            .unwrap();
        let underlying_prices = router
            .query_prices(app, underlying_keys)
            .unwrap()
            .iter()
            .map(|p| p.data.rate)
            .collect::<Vec<_>>();
        let actual_rates = router
            .query_prices(app, derivative_rate_keys)
            .unwrap()
            .iter()
            .map(|p| p.data.rate)
            .collect::<Vec<_>>();
        let expected_prices = vec![
            underlying_prices[0].multiply_ratio(actual_rates[0], 10u128.pow(18)),
            underlying_prices[1].multiply_ratio(actual_rates[1], 10u128.pow(18)),
            underlying_prices[2].multiply_ratio(actual_rates[2], 10u128.pow(18)),
        ];
        let actual_prices = router
            .query_prices(app, keys.clone())
            .unwrap()
            .iter()
            .map(|p| p.data.rate)
            .collect::<Vec<_>>();
        assert_eq!(expected_prices, actual_prices);

        let actual_price = router.query_price(app, keys[0].clone()).unwrap();
        assert_eq!(expected_prices[0], actual_price.data.rate);
    }
}
