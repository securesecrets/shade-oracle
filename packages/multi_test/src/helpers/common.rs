use super::router::OracleRouterHelper;
use super::*;
use cosmwasm_std::Timestamp;
use multi_test_helpers::admin_auth::AdminAuthHelper;
use shade_oracles::interfaces::providers::mock::{BandExecuteMsg, BandMockPrice, MockPrice};
use shade_oracles::interfaces::providers::{self, Provider, RawProvider};
use shade_toolkit::{multi_test::App, AnyResult, Contract};

create_test_helper!(MockProviderHelper);
impl MockProviderHelper {
    pub fn init_band(
        sender: &User,
        app: &mut App,
        initial_prices: Vec<(String, String, Uint128)>,
        admin_auth: RawContract,
        quote_symbol: Option<String>,
    ) -> Self {
        let msg = providers::mock::BandInstantiateMsg {
            initial_prices,
            admin_auth,
            quote_symbol,
        };
        Self(
            sender
                .init(app, &msg, MockBand::default(), "mock_band")
                .unwrap(),
        )
    }
    pub fn init_ojo(
        sender: &User,
        app: &mut App,
        initial_prices: Vec<(String, String, Uint256)>,
        admin_auth: RawContract,
        quote_symbol: Option<String>,
    ) -> Self {
        let msg = providers::mock::OjoInstantiateMsg {
            initial_prices,
            admin_auth,
            quote_symbol,
        };
        Self(
            sender
                .init(app, &msg, MockOjo::default(), "mock_ojo")
                .unwrap(),
        )
    }
    pub fn update_band_prices(
        &self,
        sender: &User,
        app: &mut App,
        prices: HashMap<String, Uint128>,
        last_updated_time: Option<u64>,
    ) {
        let mut mock_prices = vec![];
        for (sym, price) in prices {
            mock_prices.push(BandMockPrice {
                base_symbol: sym,
                quote_symbol: "USD".into(),
                rate: price,
                last_updated: last_updated_time,
            });
        }
        sender
            .exec(
                app,
                &providers::mock::BandExecuteMsg::SetPrices(mock_prices),
                &self.0,
            )
            .unwrap();
    }
    pub fn update_ojo_prices(
        &self,
        sender: &User,
        app: &mut App,
        prices: HashMap<String, Uint256>,
        last_updated_time: Option<u64>,
    ) {
        let mut mock_prices = vec![];
        for (sym, price) in prices {
            mock_prices.push(MockPrice {
                base_symbol: sym,
                quote_symbol: "USD".into(),
                rate: price,
                last_updated: last_updated_time,
            });
        }
        sender
            .exec(
                app,
                &providers::mock::OjoExecuteMsg::SetPrices(mock_prices),
                &self.0,
            )
            .unwrap();
    }
}

#[derive(Clone)]
pub struct OracleCore {
    pub provider: MockProviderHelper,
    pub router: OracleRouterHelper,
    pub admin_auth: AdminAuthHelper,
    pub superadmin: User,
}

impl OracleCore {
    pub fn new(
        provider: MockProviderHelper,
        router: OracleRouterHelper,
        admin_auth: AdminAuthHelper,
        superadmin: User,
    ) -> Self {
        OracleCore {
            provider,
            router,
            admin_auth,
            superadmin,
        }
    }
    /// Initializes the core dependencies for testing all oracles which are
    /// provider, proxy provider, router, and the admin auth contract. Then, it updates the prices in provider
    /// based off the prices argument with them being quoted in "USD".
    pub fn setup(
        app: &mut App,
        admin: &User,
        prices: HashMap<String, Uint128>,
        provider: Option<MockProviderHelper>,
        oracle_router: Option<OracleRouterHelper>,
        admin_auth: Option<AdminAuthHelper>,
    ) -> AnyResult<Self> {
        let quote_symbol: String = "USD".into();
        let admin_auth = admin_auth.unwrap_or_else(|| AdminAuthHelper::init(app, admin, None));
        let mut initial_prices = vec![];
        // Configure mock provider prices
        for (sym, price) in prices {
            initial_prices.push((sym, quote_symbol.clone(), price));
        }

        let provider = provider.unwrap_or_else(|| {
            MockProviderHelper::init_band(
                admin,
                app,
                initial_prices,
                admin_auth.clone().into(),
                Some("USD".into()),
            )
        });

        let oracle_router = oracle_router.unwrap_or_else(|| {
            OracleRouterHelper::init(
                admin,
                app,
                &admin_auth.clone().0.into(),
                RawProvider::Band(provider.clone().into()),
                "USD",
            )
        });

        Ok(OracleCore::new(
            provider,
            oracle_router,
            admin_auth,
            admin.clone(),
        ))
    }

    pub fn update_prices(
        &self,
        app: &mut App,
        prices: HashMap<String, Uint128>,
        last_updated_time: u64,
    ) {
        self.provider
            .update_band_prices(&self.superadmin, app, prices, Some(last_updated_time));
    }

    pub fn set_keys(&self, app: &mut App, oracle: Contract, keys: Vec<String>) {
        self.router
            .set_keys(&self.superadmin, app, oracle, keys)
            .unwrap();
    }

    pub fn remove_keys(&self, app: &mut App, keys: Vec<String>) {
        self.router
            .remove_keys(&self.superadmin, app, keys)
            .unwrap();
    }

    /// Turns a vector of (key, price) into a hashmap of key to price, returning a vector of keys and the hashmap.
    pub fn create_prices_hashmap(
        prices: Vec<(impl Into<String> + Clone, impl Into<Uint128>)>,
    ) -> (Vec<String>, HashMap<String, Uint128>) {
        let mut keys = vec![];
        let prices: HashMap<String, Uint128> = prices
            .into_iter()
            .map(|(sym, p)| {
                keys.push(sym.clone().into());
                (sym.into(), p.into())
            })
            .collect();
        (keys, prices)
    }
}

/// Basic test scenario with prices, keys, core oracle dependencies, and three users.
/// Also generates a hashmap of snip20s based off the prices.
pub struct TestScenario {
    pub app: App,
    pub admin: User,
    pub bot: User,
    pub user: User,
    pub keys: Vec<String>,
    pub prices: HashMap<String, Uint128>,
    pub tokens: HashMap<String, Snip20Helper>,
    pub provider: MockProviderHelper,
    pub router: OracleRouterHelper,
    pub admin_auth: AdminAuthHelper,
}

impl TestScenario {
    // 1e-15
    pub const ERROR: Decimal256 = Decimal256::new(Uint256::from_u128(100u128));

    pub fn new(prices: Vec<(impl Into<String> + Clone, impl Into<Uint128>)>) -> Self {
        let mut app = App::default();
        app.update_block(|b| b.time = Timestamp::from_seconds(0));
        let bot = User::new("bot");
        let admin = User::new("superadmin");
        let user = User::new("user");
        let (keys, prices) = OracleCore::create_prices_hashmap(prices);
        let mut tokens = HashMap::new();
        for key in &keys {
            tokens.insert(
                key.clone(),
                Snip20Helper::init(
                    &admin,
                    &mut app,
                    key,
                    key,
                    6,
                    &admin.address,
                    &None,
                    &to_binary("wdwdadwad").unwrap(),
                    &format!("{key}-snip20"),
                ),
            );
        }
        let OracleCore {
            provider,
            router,
            admin_auth,
            superadmin,
        } = OracleCore::setup(&mut app, &admin, prices.clone(), None, None, None).unwrap();
        TestScenario {
            app,
            bot,
            tokens,
            admin: superadmin,
            user,
            provider,
            router,
            admin_auth,
            keys,
            prices,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shade_oracles::{
        interfaces::{
            providers::{
                mock::{BandExecuteMsg, Config as ProviderConfig, OjoExecuteMsg},
                BandQueryMsg, OjoQueryMsg, OjoReferenceData,
            },
            router::registry::Oracle,
        },
        unit_test_interface::prices::PricesFixture,
    };

    #[test]
    fn test_mock_providers() {
        let mut app = App::default();
        let admin = User::new("superadmin");
        let user = User::new("user");
        let admin_auth = AdminAuthHelper::init(&mut app, &admin, None);

        let prices = PricesFixture::basic_prices_2();
        let (keys, _) = OracleCore::create_prices_hashmap(prices.clone());
        let quote_symbol: String = "USD".into();
        let mut initial_prices = vec![];
        let mut initial_ojo_prices = vec![];
        // Configure mock provider prices
        for (sym, price) in prices.clone() {
            initial_prices.push((sym.to_string(), quote_symbol.clone(), Uint128::new(price)));
            initial_ojo_prices.push((
                sym.to_string(),
                quote_symbol.clone(),
                Uint256::from_u128(price),
            ));
        }

        let band = MockProviderHelper::init_band(
            &admin,
            &mut app,
            initial_prices,
            admin_auth.clone().into(),
            Some(quote_symbol.clone()),
        );

        let ojo = MockProviderHelper::init_ojo(
            &admin,
            &mut app,
            initial_ojo_prices,
            admin_auth.into(),
            Some(quote_symbol.clone()),
        );

        let mut mock_prices = vec![];
        let mut ojo_mock_prices = vec![];
        for (sym, price) in prices.clone() {
            mock_prices.push(BandMockPrice {
                base_symbol: sym.to_string(),
                quote_symbol: "USD".into(),
                rate: Uint128::new(price),
                last_updated: None,
            });
            ojo_mock_prices.push(MockPrice {
                base_symbol: sym.to_string(),
                quote_symbol: "USD".into(),
                rate: Uint256::from_u128(price),
                last_updated: None,
            });
        }

        // EXECUTES ARE ADMIN PROTECTED
        let band_msgs = vec![
            BandExecuteMsg::SetPrice(mock_prices[0].clone()),
            BandExecuteMsg::SetPrices(mock_prices.clone()),
            BandExecuteMsg::SetStatus(true),
            BandExecuteMsg::UpdateConfig {
                admin_auth: None,
                quote_symbol: None,
            },
        ];

        for msg in band_msgs {
            assert!(user.exec(&mut app, &msg, &band.0).is_err());
            assert!(admin.exec(&mut app, &msg, &band.0).is_ok());
        }

        let ojo_msgs = vec![
            OjoExecuteMsg::SetPrice(ojo_mock_prices[0].clone()),
            OjoExecuteMsg::SetPrices(ojo_mock_prices.clone()),
            OjoExecuteMsg::SetStatus(true),
            OjoExecuteMsg::UpdateConfig {
                admin_auth: None,
                quote_symbol: None,
            },
        ];

        for msg in ojo_msgs {
            assert!(user.exec(&mut app, &msg, &ojo.0).is_err());
            assert!(admin.exec(&mut app, &msg, &ojo.0).is_ok());
        }

        // STATUS DISABLE WORKS
        let band_queries = vec![
            BandQueryMsg::GetPrice {
                key: keys[0].clone(),
            },
            BandQueryMsg::GetPrices { keys: keys.clone() },
            BandQueryMsg::GetReferenceData {
                base_symbol: keys[0].clone(),
                quote_symbol: quote_symbol.clone(),
            },
            BandQueryMsg::GetReferenceDataBulk {
                base_symbols: keys.clone(),
                quote_symbols: keys.clone().iter().map(|_| quote_symbol.clone()).collect(),
            },
        ];
        assert!(admin
            .exec(&mut app, &BandExecuteMsg::SetStatus(false), &band.0)
            .is_ok());
        let config: ProviderConfig = BandQueryMsg::GetConfig {}
            .test_query(&band.0, &app)
            .unwrap();
        assert!(!config.enabled);
        for msg in band_queries.clone() {
            assert!(msg.test_query::<PriceResponse>(&band.0, &app).is_err());
        }
        assert!(admin
            .exec(&mut app, &BandExecuteMsg::SetStatus(true), &band.0)
            .is_ok());
        let config: ProviderConfig = BandQueryMsg::GetConfig {}
            .test_query(&band.0, &app)
            .unwrap();
        assert!(config.enabled);
        assert_eq!(
            Uint256::from_u128(prices[0].1),
            band_queries[0]
                .test_query::<PriceResponse>(&band.0, &app)
                .unwrap()
                .data
                .rate
        );
        assert_eq!(
            Uint256::from_u128(prices[1].1),
            band_queries[1]
                .test_query::<PricesResponse>(&band.0, &app)
                .unwrap()[1]
                .data
                .rate
        );

        let ojo_queries = vec![
            OjoQueryMsg::GetPrice {
                key: keys[0].clone(),
            },
            OjoQueryMsg::GetPrices { keys: keys.clone() },
            OjoQueryMsg::GetReferenceData {
                symbol_pair: (keys[0].clone(), quote_symbol.clone()),
            },
            OjoQueryMsg::GetReferenceDataBulk {
                symbol_pairs: keys
                    .clone()
                    .iter()
                    .map(|k| (k.clone(), quote_symbol.clone()))
                    .collect(),
            },
            OjoQueryMsg::GetMedianReferenceData {
                symbol_pair: (keys[0].clone(), quote_symbol.clone()),
            },
            OjoQueryMsg::GetMedianReferenceDataBulk {
                symbol_pairs: keys
                    .clone()
                    .iter()
                    .map(|k| (k.clone(), quote_symbol.clone()))
                    .collect(),
            },
        ];
        assert!(admin
            .exec(&mut app, &OjoExecuteMsg::SetStatus(false), &ojo.0)
            .is_ok());
        let config: ProviderConfig = OjoQueryMsg::GetConfig {}.test_query(&ojo.0, &app).unwrap();
        assert!(!config.enabled);
        for msg in ojo_queries.clone() {
            assert!(msg.test_query::<PriceResponse>(&ojo.0, &app).is_err());
        }
        assert!(admin
            .exec(&mut app, &OjoExecuteMsg::SetStatus(true), &ojo.0)
            .is_ok());
        let config: ProviderConfig = OjoQueryMsg::GetConfig {}.test_query(&ojo.0, &app).unwrap();
        assert!(config.enabled);
        assert_eq!(
            Uint256::from_u128(prices[0].1),
            ojo_queries[0]
                .test_query::<PriceResponse>(&ojo.0, &app)
                .unwrap()
                .data
                .rate
        );
        assert_eq!(
            Uint256::from_u128(prices[1].1),
            ojo_queries[1]
                .test_query::<PricesResponse>(&ojo.0, &app)
                .unwrap()[1]
                .data
                .rate
        );
        assert_eq!(
            Uint256::from_u128(prices[0].1),
            ojo_queries[2]
                .test_query::<OjoReferenceData>(&ojo.0, &app)
                .unwrap()
                .rate
        );
        assert_eq!(
            Uint256::from_u128(prices[1].1),
            ojo_queries[3]
                .test_query::<Vec<OjoReferenceData>>(&ojo.0, &app)
                .unwrap()[1]
                .rate
        );
        assert_eq!(
            Uint256::from_u128(prices[0].1),
            ojo_queries[4]
                .test_query::<OjoReferenceData>(&ojo.0, &app)
                .unwrap()
                .rate
        );
        assert_eq!(
            Uint256::from_u128(prices[1].1),
            ojo_queries[5]
                .test_query::<Vec<OjoReferenceData>>(&ojo.0, &app)
                .unwrap()[1]
                .rate
        );
    }
}
