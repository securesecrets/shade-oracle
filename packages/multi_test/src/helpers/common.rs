use super::router::OracleRouterHelper;
use super::*;
use multi_test_helpers::admin_auth::AdminAuthHelper;
use shade_oracles::interfaces::band::MockPrice;
use shade_oracles::interfaces::band::{self};
use shade_protocol::{multi_test::App, AnyResult, Contract};

create_test_helper!(BandHelper);
impl BandHelper {
    pub fn init(
        sender: &User,
        app: &mut App,
        initial_prices: Vec<(String, String, Uint128)>,
        admin_auth: RawContract,
        quote_symbol: Option<String>,
    ) -> Self {
        let msg = band::InstantiateMsg {
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
    pub fn update_prices(
        &self,
        sender: &User,
        app: &mut App,
        prices: HashMap<String, Uint128>,
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
            .exec(app, &band::ExecuteMsg::SetPrices(mock_prices), &self.0)
            .unwrap();
    }
}

#[derive(Clone)]
pub struct OracleCore {
    pub band: BandHelper,
    pub router: OracleRouterHelper,
    pub admin_auth: AdminAuthHelper,
    pub superadmin: User,
}

impl OracleCore {
    pub fn new(
        band: BandHelper,
        router: OracleRouterHelper,
        admin_auth: AdminAuthHelper,
        superadmin: User,
    ) -> Self {
        OracleCore {
            band,
            router,
            admin_auth,
            superadmin,
        }
    }
    /// Initializes the core dependencies for testing all oracles which are
    /// band, proxy band, router, and the admin auth contract. Then, it updates the prices in band
    /// based off the prices argument with them being quoted in "USD".
    pub fn setup(
        app: &mut App,
        admin: &User,
        prices: HashMap<String, Uint128>,
        band: Option<BandHelper>,
        oracle_router: Option<OracleRouterHelper>,
        admin_auth: Option<AdminAuthHelper>,
    ) -> AnyResult<Self> {
        let quote_symbol: String = "USD".into();
        let admin_auth = admin_auth.unwrap_or_else(|| AdminAuthHelper::init(app, admin, None));
        let mut initial_prices = vec![];
        // Configure mock band prices
        for (sym, price) in prices {
            initial_prices.push((sym, quote_symbol.clone(), price));
        }

        let band = band.unwrap_or_else(|| {
            BandHelper::init(
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
                &band.clone().0.into(),
                "USD",
            )
        });

        Ok(OracleCore::new(
            band,
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
        self.band
            .update_prices(&self.superadmin, app, prices, Some(last_updated_time));
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
    pub band: BandHelper,
    pub router: OracleRouterHelper,
    pub admin_auth: AdminAuthHelper,
}

impl TestScenario {
    pub fn new(prices: Vec<(impl Into<String> + Clone, impl Into<Uint128>)>) -> Self {
        let mut app = App::default();
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
            band,
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
            band,
            router,
            admin_auth,
            keys,
            prices,
        }
    }
}
