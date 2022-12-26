use crate::multi::{MockBand, OracleRouter};
use shade_multi_test::multi::admin::init_admin_auth;
use shade_oracles::core::Query;
use shade_oracles::interfaces::band::MockPrice;
use shade_oracles::interfaces::common::{PriceResponse, PricesResponse};
use shade_oracles::interfaces::router::msg::QueryMsg;
use shade_oracles::interfaces::router::registry::{RegistryOperation, UpdateConfig};
use shade_oracles::interfaces::{
    band::{self},
    router::{self},
};
use shade_protocol::multi_test::AppResponse;
use shade_protocol::{
    c_std::{Addr, ContractInfo, StdResult, Uint128},
    multi_test::App,
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable},
    AnyResult, Contract,
};
use std::collections::HashMap;

pub struct BandHelper(pub ContractInfo);

impl BandHelper {
    pub fn update_prices(
        &self,
        sender: &Addr,
        app: &mut App,
        prices: HashMap<String, Uint128>,
        last_updated_time: u64,
    ) {
        for (sym, price) in prices {
            band::ExecuteMsg::SetPrice {
                price: MockPrice {
                    base_symbol: sym,
                    quote_symbol: "USD".into(),
                    rate: price,
                    last_updated: Some(last_updated_time),
                },
            }
            .test_exec(&self.0, app, sender.clone(), &[])
            .unwrap();
        }
    }
}

pub struct OracleRouterHelper(pub ContractInfo);

impl OracleRouterHelper {
    pub fn update_registry(
        &self,
        sender: &Addr,
        app: &mut App,
        operation: RegistryOperation,
    ) -> AnyResult<AppResponse> {
        router::msg::ExecuteMsg::UpdateRegistry(operation).test_exec(
            &self.0,
            app,
            sender.clone(),
            &[],
        )
    }
    pub fn set_keys(
        &self,
        sender: &Addr,
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
        sender: &Addr,
        app: &mut App,
        keys: Vec<String>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::RemoveKeys { keys })
    }

    pub fn protect_keys(
        &self,
        sender: &Addr,
        app: &mut App,
        infos: Vec<router::registry::ProtectedKeyInfo>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::SetProtection { infos })
    }
    pub fn remove_key_protections(
        &self,
        sender: &Addr,
        app: &mut App,
        keys: Vec<String>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::RemoveProtection { keys })
    }
    pub fn query_price(&self, app: &App, key: String) -> StdResult<PriceResponse> {
        QueryMsg::GetPrice { key }.test_query(&self.0, app)
    }
    pub fn query_prices(&self, app: &App, keys: Vec<String>) -> StdResult<PricesResponse> {
        QueryMsg::GetPrices { keys }.test_query(&self.0, app)
    }
}

pub struct OracleCore {
    pub band: BandHelper,
    pub router: OracleRouterHelper,
    pub admin_auth: ContractInfo,
    pub superadmin: Addr,
}

impl OracleCore {
    pub fn new(
        band: BandHelper,
        router: OracleRouterHelper,
        admin_auth: ContractInfo,
        superadmin: Addr,
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
        admin: &Addr,
        prices: HashMap<String, Uint128>,
        band: Option<ContractInfo>,
        oracle_router: Option<ContractInfo>,
        admin_auth: Option<ContractInfo>,
    ) -> AnyResult<Self> {
        let quote_symbol = "USD".to_string();
        let superadmin = admin.clone();

        let admin_auth = admin_auth.unwrap_or_else(|| init_admin_auth(app, admin));
        let mut initial_prices = vec![];
        // Configure mock band prices
        for (sym, price) in prices {
            initial_prices.push((sym, quote_symbol.clone(), price));
        }

        let band = band.unwrap_or_else(|| {
            band::InstantiateMsg { initial_prices }
                .test_init(MockBand::default(), app, admin.clone(), "band", &[])
                .unwrap()
        });

        let oracle_router = oracle_router.unwrap_or_else(|| {
            router::msg::InstantiateMsg {
                admin_auth: admin_auth.clone().into(),
                band: band.clone().into(),
                quote_symbol: quote_symbol.clone(),
            }
            .test_init(
                OracleRouter::default(),
                app,
                admin.clone(),
                "oracle_router",
                &[],
            )
            .unwrap()
        });

        router::msg::ExecuteMsg::UpdateConfig(UpdateConfig {
            admin_auth: None,
            band: None,
            quote_symbol: None,
        })
        .test_exec(&oracle_router, app, admin.clone(), &[])
        .unwrap();

        Ok(OracleCore::new(
            BandHelper(band),
            OracleRouterHelper(oracle_router),
            admin_auth,
            superadmin,
        ))
    }

    pub fn update_prices(
        &self,
        app: &mut App,
        prices: HashMap<String, Uint128>,
        last_updated_time: u64,
    ) {
        self.band
            .update_prices(&self.superadmin, app, prices, last_updated_time);
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
}
