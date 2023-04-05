use super::*;
use shade_oracles::{
    interfaces::{
        providers::RawProvider,
        router::{msg::*, registry::ProtectedKeyInfo},
    },
    status::ContractStatus,
};

create_test_helper!(OracleRouterHelper);

impl OracleRouterHelper {
    pub fn init(
        user: &User,
        app: &mut App,
        admin_auth: &Contract,
        provider: RawProvider,
        quote_symbol: &str,
    ) -> Self {
        let contract = user
            .init(
                app,
                &InstantiateMsg {
                    admin_auth: admin_auth.clone().into(),
                    provider,
                    quote_symbol: quote_symbol.to_string(),
                },
                OracleRouter::default(),
                "oracle_router",
            )
            .unwrap();
        Self(contract)
    }
    pub fn update_registry(
        &self,
        sender: &User,
        app: &mut App,
        operation: RegistryOperation,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::UpdateRegistry(operation), &self.0)
    }

    pub fn batch_update_registry(
        &self,
        sender: &User,
        app: &mut App,
        operations: &[RegistryOperation],
    ) -> AnyResult<AppResponse> {
        sender.exec(
            app,
            &ExecuteMsg::BatchUpdateRegistry(operations.to_vec()),
            &self.0,
        )
    }

    pub fn set_status(
        &self,
        sender: &User,
        app: &mut App,
        status: ContractStatus,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::SetStatus(status), &self.0)
    }

    pub fn update_config(
        &self,
        sender: &User,
        app: &mut App,
        config: UpdateConfig,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::UpdateConfig(config), &self.0)
    }

    pub fn set_keys(
        &self,
        sender: &User,
        app: &mut App,
        oracle: Contract,
        keys: Vec<String>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(
            sender,
            app,
            RegistryOperation::SetKeys {
                oracle: oracle.into(),
                keys,
            },
        )
    }

    pub fn remove_keys(
        &self,
        sender: &User,
        app: &mut App,
        keys: Vec<String>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::RemoveKeys { keys })
    }

    pub fn set_protection(
        &self,
        sender: &User,
        app: &mut App,
        infos: Vec<ProtectedKeyInfo>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::SetProtection { infos })
    }
    pub fn remove_key_protections(
        &self,
        sender: &User,
        app: &mut App,
        keys: Vec<String>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::RemoveProtection { keys })
    }
    pub fn update_protected_keys(
        &self,
        sender: &User,
        app: &mut App,
        updates: Vec<(String, Uint256)>,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::UpdateProtectedKeys(updates), &self.0)
    }
    pub fn query_config(&self, app: &App) -> StdResult<ConfigResponse> {
        QueryMsg::GetConfig {}.test_query(&self.0, app)
    }
    pub fn query_oracle(&self, app: &App, key: &str) -> StdResult<OracleResponse> {
        QueryMsg::GetOracle {
            key: key.to_string(),
        }
        .test_query(&self.0, app)
    }

    pub fn query_price(&self, app: &App, key: String) -> StdResult<PriceResponse> {
        QueryMsg::GetPrice { key }.test_query(&self.0, app)
    }

    pub fn query_oracles(&self, app: &App, keys: Vec<String>) -> StdResult<OraclesResponse> {
        QueryMsg::GetOracles { keys }.test_query(&self.0, app)
    }

    pub fn query_prices(&self, app: &App, keys: Vec<String>) -> StdResult<PricesResponse> {
        QueryMsg::GetPrices { keys }.test_query(&self.0, app)
    }

    pub fn query_keys(&self, app: &App) -> StdResult<KeysResponse> {
        QueryMsg::GetKeys {}.test_query(&self.0, app)
    }

    pub fn query_protected_keys(&self, app: &App) -> StdResult<ProtectedKeysResponse> {
        QueryMsg::GetProtectedKeys {}.test_query(&self.0, app)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use multi_test_helpers::Asserter;
    use shade_oracles::{
        core::admin::helpers::AdminPermissions, unit_test_interface::prices::PricesFixture,
    };

    /// Tests set protection, remove key protection, update protection, query protected keys.
    #[test]
    fn protected_query_tests() {
        let prices = vec![("USD", 1_00 * 10u128.pow(16))];

        let TestScenario {
            mut app,
            router,
            admin,
            provider,
            admin_auth,
            ..
        } = TestScenario::new(prices);
        let user = admin;
        let app = &mut app;

        let usd_protection = ProtectedKeyInfo::new(
            "USD".to_string(),
            Decimal256::percent(4),
            Uint256::from_u128(1_00 * 10u128.pow(16)),
        );
        let protection = vec![usd_protection.clone()];

        // Set initial oracle key protection.
        router
            .set_protection(&user, app, protection.clone())
            .unwrap();
        assert!(router.query_price(app, "USD".to_string()).is_ok());
        assert!(router.query_prices(app, vec!["USD".to_string()]).is_ok());
        assert!(router
            .query_protected_keys(app)
            .unwrap()
            .contains(&usd_protection));

        // Set price to 1.05 which is greater than protection deviation of 4% from 1.00 so should fail.
        let prices = vec![("USD", 1_05 * 10u128.pow(16))];
        let prices = OracleCore::create_prices_hashmap(prices).1;
        provider.update_band_prices(&user, app, prices, Some(app.block_info().time.seconds()));
        assert!(router.query_price(app, "USD".to_string()).is_err());
        assert!(router.query_prices(app, vec!["USD".to_string()]).is_err());

        // Remove protection and see it works again.
        router
            .remove_key_protections(&user, app, vec!["USD".to_string()])
            .unwrap();
        assert!(router.query_price(app, "USD".to_string()).is_ok());
        assert!(router.query_prices(app, vec!["USD".to_string()]).is_ok());
        assert!(router.query_protected_keys(app).unwrap().is_empty());

        // Set protection again.
        router.set_protection(&user, app, protection).unwrap();

        // Create a bot to update the protection price so that its equal to the current price.
        let bot = User::new("bot");
        let resp = router.update_protected_keys(
            &bot,
            app,
            vec![("USD".to_string(), Uint256::from_u128(1_05 * 10u128.pow(16)))],
        );
        assert!(resp.is_err());

        admin_auth.register_admin(&user, app, bot.clone().into());
        admin_auth.grant_access(
            &user,
            app,
            bot.clone().into(),
            vec![AdminPermissions::OraclesPriceBot.into_string()],
        );

        let resp = router.update_protected_keys(
            &bot,
            app,
            vec![("USD".to_string(), Uint256::from_u128(1_05 * 10u128.pow(16)))],
        );
        assert!(resp.is_ok());
        assert!(router.query_price(app, "USD".to_string()).is_ok());
        assert!(router.query_prices(app, vec!["USD".to_string()]).is_ok());
    }

    #[rstest]
    #[case(PricesFixture::basic_prices_1())]
    #[case(PricesFixture::basic_prices_2())]
    fn basic_query_test(#[case] prices: Vec<(&str, u128)>) {
        let TestScenario {
            app,
            router,
            keys,
            prices,
            ..
        } = TestScenario::new(prices);
        let resp = router.query_prices(&app, keys).unwrap();
        for price in resp {
            let p: Uint256 = (*prices.get(price.key()).unwrap()).into();
            assert_eq!(price.data.rate, p);
        }
    }

    #[test]
    fn duplicate_symbol_query_test() {
        let TestScenario {
            app,
            router,
            keys,
            prices,
            ..
        } = TestScenario::new(PricesFixture::basic_prices_1());
        let keys = vec![keys[0].clone(), keys[0].clone(), keys[1].clone()];
        let resp = router.query_prices(&app, keys).unwrap();
        for price in resp {
            let p: Uint256 = (*prices.get(price.key()).unwrap()).into();
            assert_eq!(price.data.rate, p);
        }
    }

    #[test]
    fn registry_tests() {
        let prices = PricesFixture::basic_prices_2();
        let test_prices = prices.clone();
        let random = User::new("random");
        let TestScenario {
            mut app,
            router,
            admin,
            keys,
            provider,
            ..
        } = TestScenario::new(prices);
        let user = admin;

        assert!(router
            .set_status(&random, &mut app, ContractStatus::Deprecated)
            .is_err());
        router
            .set_status(&user, &mut app, ContractStatus::Deprecated)
            .unwrap();
        assert!(router
            .query_price(&app, test_prices[0].0.to_string())
            .is_err());

        // Update config test.
        router
            .update_config(
                &user,
                &mut app,
                UpdateConfig {
                    admin_auth: None,
                    provider: None,
                    quote_symbol: Some("JPY".to_string()),
                },
            )
            .unwrap();
        assert!(router.query_config(&app).unwrap().config.quote_symbol == *"JPY");

        router
            .set_status(&user, &mut app, ContractStatus::Frozen)
            .unwrap();
        assert!(router
            .update_config(
                &user,
                &mut app,
                UpdateConfig {
                    admin_auth: None,
                    provider: None,
                    quote_symbol: Some("USD".to_string())
                }
            )
            .is_err());

        router
            .set_status(&user, &mut app, ContractStatus::Normal)
            .unwrap();
        assert!(router
            .update_config(
                &user,
                &mut app,
                UpdateConfig {
                    admin_auth: None,
                    provider: None,
                    quote_symbol: Some("USD".to_string())
                }
            )
            .is_ok());

        router
            .set_keys(&user, &mut app, provider.clone().into(), keys.clone())
            .unwrap();
        let oracles_resp = router.query_oracles(&app, keys.clone()).unwrap();
        let keys_resp = router.query_keys(&app).unwrap();
        assert_eq!(keys.len(), keys_resp.len());
        assert_eq!(keys.len(), oracles_resp.len());
        for oracle in oracles_resp {
            assert_eq!(oracle.oracle, provider.clone().into());
        }
        Asserter::equal_vecs(&keys, &keys_resp);

        let keys_to_remove = vec![test_prices[0].0.to_string(), test_prices[1].0.to_string()];
        router
            .remove_keys(&user, &mut app, keys_to_remove.clone())
            .unwrap();
        let oracles_resp = router.query_oracles(&app, keys.clone()).unwrap();
        let keys_resp = router.query_keys(&app).unwrap();
        assert_eq!(keys.len() - 2, keys_resp.len());
        for oracle in oracles_resp {
            if oracle.key == test_prices[0].0 || oracle.key == test_prices[1].0 {
                assert_eq!(oracle.oracle, router.clone().into());
            } else {
                assert_eq!(oracle.oracle, provider.clone().into());
            }
        }
        assert!(
            !keys_resp.contains(&test_prices[0].0.to_string())
                && !keys_resp.contains(&test_prices[1].0.to_string())
        );
        let oracle = router.query_oracle(&app, &keys[0].clone()).unwrap();
        assert_eq!(oracle.oracle, router.clone().into());

        let operations = vec![
            RegistryOperation::SetKeys {
                oracle: provider.clone().into(),
                keys: keys_to_remove.clone(),
            },
            RegistryOperation::RemoveKeys {
                keys: keys_to_remove.clone(),
            },
        ];
        router
            .batch_update_registry(&user, &mut app, &operations)
            .unwrap();
        let oracle = router.query_oracle(&app, &keys[0].clone()).unwrap();
        assert_eq!(oracle.oracle, router.clone().into());
    }
}
