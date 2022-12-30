use super::*;
use shade_oracles::interfaces::router::{
    msg::*,
    registry::{ProtectedKeyInfo, RegistryOperation},
};

create_test_helper!(OracleRouterHelper);
impl OracleRouterHelper {
    pub fn init(
        user: &User,
        app: &mut App,
        admin_auth: &Contract,
        band: &Contract,
        quote_symbol: &str,
    ) -> Self {
        let contract = user
            .init(
                app,
                &InstantiateMsg {
                    admin_auth: admin_auth.clone().into(),
                    band: band.clone().into(),
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
    pub fn query_price(&self, app: &App, key: String) -> StdResult<PriceResponse> {
        QueryMsg::GetPrice { key }.test_query(&self.0, app)
    }
    pub fn query_prices(&self, app: &App, keys: Vec<String>) -> StdResult<PricesResponse> {
        QueryMsg::GetPrices { keys }.test_query(&self.0, app)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use shade_oracles::core::admin::helpers::AdminPermissions;

    pub fn basic_prices_1() -> Vec<(&'static str, u128)> {
        vec![
            ("USD", 10u128.pow(18)),         // $1
            ("SILK", 1_05 * 10u128.pow(16)), // $1.05
        ]
    }

    pub fn basic_prices_2() -> Vec<(&'static str, u128)> {
        vec![
            ("USD", 1_00 * 10u128.pow(16)),      // $1
            ("BTC", 29_398_20 * 10u128.pow(14)), // $29398.2
            ("ETH", 1_831_26 * 10u128.pow(14)),  // $1831.26
            ("XAU", 1_852_65 * 10u128.pow(14)),  // $1852.65
        ]
    }

    #[test]
    fn protected_query_admin_tests() {
        let mut keys = vec![];
        let prices = vec![("USD", 1_00 * 10u128.pow(16))];
        let prices: HashMap<String, Uint128> = prices
            .into_iter()
            .map(|(sym, p)| {
                keys.push(sym.to_string());
                (sym.to_string(), p.into())
            })
            .collect();
        let test_prices = prices.clone();
        let user = User::new("superadmin");
        let mut app = &mut App::default();

        let deps = OracleCore::setup(app, &user, prices, None, None, None).unwrap();
        let band = deps.band;
    }

    #[test]
    fn registry_tests() {
        let prices = vec![("USD", 1_00 * 10u128.pow(16))];
        let (keys, prices) = OracleCore::create_prices_hashmap(prices);
        let test_prices = prices.clone();
        let user = User::new("superadmin");
        let mut app = &mut App::default();

        let deps = OracleCore::setup(app, &user, prices, None, None, None).unwrap();
        let band = deps.band;
        let router = deps.router;

        let mock_user = User::new("randomuser");
        let mock_user2 = User::new("randomuser2");
    }

    #[rstest]
    fn protected_query_tests() {
        let prices = OracleCore::create_prices_hashmap(vec![("USD", 1_00 * 10u128.pow(16))]).1;
        let user = User::new("superadmin");
        let app = &mut App::default();

        let deps = OracleCore::setup(app, &user, prices, None, None, None).unwrap();
        let band = deps.band;
        let router = deps.router;
        router
            .set_protection(
                &user,
                app,
                vec![ProtectedKeyInfo::new(
                    "USD".to_string(),
                    Decimal256::percent(4),
                    Uint256::from_u128(1_00 * 10u128.pow(16)),
                )],
            )
            .unwrap();
        assert!(router.query_price(app, "USD".to_string()).is_ok());
        assert!(router.query_prices(app, vec!["USD".to_string()]).is_ok());
        let prices = vec![("USD", 1_05 * 10u128.pow(16))];
        let prices = OracleCore::create_prices_hashmap(prices).1;
        band.update_prices(&user, app, prices, Some(app.block_info().time.seconds()));
        assert!(router.query_price(app, "USD".to_string()).is_err());
        assert!(router.query_prices(app, vec!["USD".to_string()]).is_err());

        let bot = User::new("bot");
        let resp = router.update_protected_keys(
            &bot,
            app,
            vec![("USD".to_string(), Uint256::from_u128(1_05 * 10u128.pow(16)))],
        );
        assert!(resp.is_err());

        deps.admin_auth
            .register_admin(&user, app, bot.clone().into());
        deps.admin_auth.grant_access(
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
    #[case(basic_prices_1())]
    #[case(basic_prices_2())]
    fn basic_query_test(#[case] prices: Vec<(&str, u128)>) {
        let mut keys = vec![];
        let prices: HashMap<String, Uint128> = prices
            .into_iter()
            .map(|(sym, p)| {
                keys.push(sym.to_string());
                (sym.to_string(), p.into())
            })
            .collect();
        let test_prices = prices.clone();
        let user = User::new("superadmin");
        let mut app = App::default();

        let deps = OracleCore::setup(&mut app, &user, prices, None, None, None).unwrap();
        let resp = deps.router.query_prices(&app, keys).unwrap();
        for price in resp {
            assert_eq!(&price.data.rate, test_prices.get(price.key()).unwrap());
        }
    }
}
