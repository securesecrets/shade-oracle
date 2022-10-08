use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

use crate::multi::{MockBand, OracleRouter, ProxyBandOracle};
use shade_multi_test::multi::admin::init_admin_auth;
use shade_oracles::core::Query;
use shade_oracles::interfaces::router::registry::UpdateConfig;
use shade_oracles::{
    common::InstantiateCommonConfig,
    interfaces::{
        band::{self, proxy},
        router::{self},
    },
};
use shade_protocol::c_std::Coin;
use shade_protocol::multi_test::AppResponse;
use shade_protocol::serde::de::DeserializeOwned;
use shade_protocol::{
    c_std::{Addr, ContractInfo, StdResult, Uint128},
    multi_test::App,
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable},
    AnyResult, Contract,
};

#[derive(Clone)]
/// Allows an instance of the Multitest App to be shared across different structs.
pub struct SharedApp {
    app: Rc<RefCell<App>>,
}

impl SharedApp {
    pub fn new(app: App) -> Self {
        SharedApp {
            app: Rc::new(RefCell::new(app)),
        }
    }
    pub fn get_mut(&self) -> RefMut<App> {
        self.app.borrow_mut()
    }
    pub fn get(&self) -> Ref<App> {
        self.app.borrow()
    }
    pub fn query<T: DeserializeOwned>(
        &self,
        msg: &impl Query,
        contract: &ContractInfo,
    ) -> StdResult<T> {
        msg.test_query(contract, &self.get())
    }
    pub fn init(
        &self,
        sender: &Addr,
        msg: &impl InstantiateCallback,
        testable: impl MultiTestable,
        label: &str,
        send_funds: &[Coin],
    ) -> AnyResult<ContractInfo> {
        msg.test_init(
            testable,
            &mut self.get_mut(),
            sender.clone(),
            label,
            send_funds,
        )
    }
    pub fn exec(
        &self,
        sender: &Addr,
        msg: &(impl ExecuteCallback + std::fmt::Debug),
        contract: &ContractInfo,
    ) -> AnyResult<AppResponse> {
        msg.test_exec(contract, &mut self.get_mut(), sender.clone(), &[])
    }
    pub fn exec_with_funds(
        &self,
        sender: &Addr,
        msg: &(impl ExecuteCallback + std::fmt::Debug),
        contract: &ContractInfo,
        send_funds: &[Coin],
    ) -> AnyResult<AppResponse> {
        msg.test_exec(contract, &mut self.get_mut(), sender.clone(), send_funds)
    }
}

pub struct OracleCore {
    pub deps: HashMap<OracleDeps, Contract>,
    pub app: SharedApp,
    pub superadmin: Addr,
}

#[derive(Hash, PartialEq, Eq)]
pub enum OracleDeps {
    Band,
    ProxyBand,
    OracleRouter,
    AdminAuth,
}

impl OracleCore {
    pub fn get(&self, deps: OracleDeps) -> ContractInfo {
        self.deps.get(&deps).unwrap().clone().into()
    }
    /// Initializes the core dependencies for testing all oracles which are
    /// band, proxy band, router, and the admin auth contract. Then, it updates the prices in band
    /// based off the prices argument with them being quoted in "USD".
    pub fn setup(
        app: SharedApp,
        admin: &Addr,
        prices: HashMap<String, Uint128>,
        band: Option<ContractInfo>,
        proxy_band: Option<ContractInfo>,
        oracle_router: Option<ContractInfo>,
        admin_auth: Option<ContractInfo>,
    ) -> AnyResult<Self> {
        let mut core = OracleCore {
            deps: HashMap::new(),
            app,
            superadmin: admin.clone(),
        };
        let quote_symbol = "USD".to_string();
        core.superadmin = admin.clone();

        let admin_auth =
            admin_auth.unwrap_or_else(|| init_admin_auth(&mut core.app.get_mut(), admin));

        core.deps
            .insert(OracleDeps::AdminAuth, admin_auth.clone().into());

        let band = band.unwrap_or_else(|| {
            core.app
                .init(
                    admin,
                    &band::InstantiateMsg {},
                    MockBand::default(),
                    "band",
                    &[],
                )
                .unwrap()
        });

        core.deps.insert(OracleDeps::Band, band.clone().into());

        let oracle_router = oracle_router.unwrap_or_else(|| {
            router::msg::InstantiateMsg {
                default_oracle: admin_auth.clone().into(),
                admin_auth: admin_auth.clone().into(),
                band: band.clone().into(),
                quote_symbol: quote_symbol.clone(),
            }
            .test_init(
                OracleRouter::default(),
                &mut core.app.get_mut(),
                admin.clone(),
                "oracle-router",
                &[],
            )
            .unwrap()
        });

        core.deps
            .insert(OracleDeps::OracleRouter, oracle_router.clone().into());

        let proxy_band = proxy_band.unwrap_or_else(|| {
            proxy::InstantiateMsg {
                quote_symbol: quote_symbol.clone(),
                config: InstantiateCommonConfig::new(
                    None,
                    oracle_router.clone().into(),
                    true,
                    true,
                ),
                band: band.clone().into(),
            }
            .test_init(
                ProxyBandOracle::default(),
                &mut core.app.get_mut(),
                admin.clone(),
                "proxy-band",
                &[],
            )
            .unwrap()
        });

        core.deps
            .insert(OracleDeps::ProxyBand, proxy_band.clone().into());

        router::msg::ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                admin_auth: None,
                default_oracle: Some(proxy_band.into()),
                band: None,
                quote_symbol: None,
            },
        }
        .test_exec(&oracle_router, &mut core.app.get_mut(), admin.clone(), &[])
        .unwrap();

        // Configure mock band prices
        for (sym, price) in prices {
            band::ExecuteMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: quote_symbol.clone(),
                rate: price,
                last_updated: None,
            }
            .test_exec(&band, &mut core.app.get_mut(), admin.clone(), &[])
            .unwrap();
        }

        Ok(core)
    }

    pub fn update_prices(&self, prices: HashMap<String, Uint128>, last_updated_time: u64) {
        for (sym, price) in prices {
            band::ExecuteMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: "USD".into(),
                rate: price,
                last_updated: Some(last_updated_time),
            }
            .test_exec(
                &self.get(OracleDeps::Band),
                &mut self.app.get_mut(),
                self.superadmin.clone(),
                &[],
            )
            .unwrap();
        }
    }
}
