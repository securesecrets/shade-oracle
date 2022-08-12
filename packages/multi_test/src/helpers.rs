use std::collections::HashMap;

use shade_admin_multi_test::multi::AdminAuth;
use shade_oracles::{
    common::InstantiateCommonConfig,
    interfaces::{
        band::{self, proxy},
        router::{self, UpdateConfig},
    },
};
//use shade_multi_test::multi::snip20::Snip20;
use crate::multi::{MockBand, OracleRouter, ProxyBandOracle};
use shade_protocol::{
    c_std::{Addr, ContractInfo, Uint128},
    multi_test::App,
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable},
    AnyResult,
};

pub struct OracleCore {
    pub deps: HashMap<String, ContractInfo>,
    pub superadmin: Addr,
}

#[derive(strum::Display)]
pub enum OracleDeps{
    Band,
    ProxyBand,
    OracleRouter,
    AdminAuth,
}

impl OracleCore {
    pub fn band(&self) -> ContractInfo { self.deps.get(&OracleDeps::Band.to_string()).unwrap().clone() }
    pub fn admin_auth(&self) -> ContractInfo { self.deps.get(&OracleDeps::AdminAuth.to_string()).unwrap().clone() }
    pub fn proxy_band(&self) -> ContractInfo { self.deps.get(&OracleDeps::ProxyBand.to_string()).unwrap().clone() }
    pub fn oracle_router(&self) -> ContractInfo { self.deps.get(&OracleDeps::OracleRouter.to_string()).unwrap().clone() }
    /// Initializes the core dependencies for testing all oracles which are
    /// band, proxy band, router, and the admin auth contract. Then, it updates the prices in band
    /// based off the prices argument with them being quoted in "USD".
    pub fn setup(
        mut self,
        app: &mut App, 
        prices: HashMap<String, Uint128>,
        band: Option<ContractInfo>,
        proxy_band: Option<ContractInfo>,
        oracle_router: Option<ContractInfo>,
        admin_auth: Option<ContractInfo>,

    ) -> AnyResult<Self> {
        let admin = Addr::unchecked("superadmin");
        self.superadmin = admin.clone();

        let admin_auth = admin_auth.unwrap_or_else(|| {
            shade_admin::admin::InstantiateMsg { super_admin: None }.test_init(
                AdminAuth::default(),
                app,
                admin.clone(),
                "admin-auth",
                &[],
            ).unwrap()
        });

        self.deps.insert(OracleDeps::AdminAuth.to_string(), admin_auth.clone());

        let band = band.unwrap_or_else(|| {
            band::InstantiateMsg {}.test_init(
                MockBand::default(),
                app,
                admin.clone(),
                "band",
                &[],
            ).unwrap()
        });

        self.deps.insert(OracleDeps::Band.to_string(), band.clone());

        let oracle_router = oracle_router.unwrap_or_else(|| {
            router::InstantiateMsg {
                default_oracle: admin_auth.clone().into(),
                admin_auth: admin_auth.clone().into(),
                band: band.clone().into(),
                quote_symbol: "USD".to_string(),
            }
            .test_init(
                OracleRouter::default(),
                app,
                admin.clone(),
                "oracle-router",
                &[],
            ).unwrap()
        });

        self.deps.insert(OracleDeps::OracleRouter.to_string(), oracle_router.clone());


        let proxy_band= proxy_band.unwrap_or_else(|| { proxy::InstantiateMsg {
            quote_symbol: "USD".to_string(),
            config: InstantiateCommonConfig::new(None, oracle_router.clone().into(), true, true),
            band: band.clone().into(),
        }
        .test_init(
            ProxyBandOracle::default(),
            app,
            admin.clone(),
            "proxy-band",
            &[],
        ).unwrap() });

        self.deps.insert(OracleDeps::ProxyBand.to_string(), proxy_band.clone());

        router::ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                admin_auth: None,
                default_oracle: Some(proxy_band.clone().into()),
                band: None,
                quote_symbol: None,
                enabled: None,
            },
        }
        .test_exec(&oracle_router, app, admin.clone(), &[])
        .unwrap();

        // Configure mock band prices
        for (sym, price) in prices {
            band::ExecuteMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: "USD".to_string(),
                rate: price,
                last_updated: None,
            }
            .test_exec(&band, app, admin.clone(), &[])
            .unwrap();
        }

        Ok(self)
    }

    pub fn update_prices(
        &self,
        app: &mut App,
        prices: HashMap<String, Uint128>,
        last_updated_time: u64,
    ) {
        for (sym, price) in prices {
            band::ExecuteMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: "USD".to_string(),
                rate: price,
                last_updated: Some(last_updated_time),
            }
            .test_exec(&self.band(), app, self.superadmin.clone(), &[])
            .unwrap();
        }
    }
}
