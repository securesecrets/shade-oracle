use crate::multi::{MockBand, OracleRouter};
use shade_multi_test::multi::admin::init_admin_auth;
use shade_oracles::interfaces::router::registry::UpdateConfig;
use shade_oracles::interfaces::{
    band::{self},
    router::{self},
};
use shade_protocol::{
    c_std::{Addr, ContractInfo, Uint128},
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
            band::ExecuteMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: "USD".into(),
                rate: price,
                last_updated: Some(last_updated_time),
            }
            .test_exec(&self.0, app, sender.clone(), &[])
            .unwrap();
        }
    }
}

pub struct OracleRouterHelper(pub ContractInfo);

impl OracleRouterHelper {
    pub fn add_oracle(&self, sender: &Addr, app: &mut App, oracle: Contract, key: String) {
        router::msg::ExecuteMsg::UpdateRegistry {
            operation: router::registry::RegistryOperation::Add { oracle, key },
        }
        .test_exec(&self.0, app, sender.clone(), &[])
        .unwrap();
    }

    pub fn remove_oracle(&self, sender: &Addr, app: &mut App, key: String) {
        router::msg::ExecuteMsg::UpdateRegistry {
            operation: router::registry::RegistryOperation::Remove { key },
        }
        .test_exec(&self.0, app, sender.clone(), &[])
        .unwrap();
    }

    pub fn replace_oracle(&self, sender: &Addr, app: &mut App, oracle: Contract, key: String) {
        router::msg::ExecuteMsg::UpdateRegistry {
            operation: router::registry::RegistryOperation::Replace { oracle, key },
        }
        .test_exec(&self.0, app, sender.clone(), &[])
        .unwrap();
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

        router::msg::ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                admin_auth: None,
                band: None,
                quote_symbol: None,
            },
        }
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

    pub fn add_oracle(&self, app: &mut App, oracle: Contract, key: String) {
        self.router.add_oracle(&self.superadmin, app, oracle, key);
    }

    pub fn remove_oracle(&self, app: &mut App, key: String) {
        self.router.remove_oracle(&self.superadmin, app, key);
    }

    pub fn replace_oracle(&self, app: &mut App, oracle: Contract, key: String) {
        self.router
            .replace_oracle(&self.superadmin, app, oracle, key);
    }
}
