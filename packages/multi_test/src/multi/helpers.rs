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

pub struct OracleCore {
    pub deps: HashMap<OracleDeps, Contract>,
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
        app: &mut App,
        admin: &Addr,
        prices: HashMap<String, Uint128>,
        band: Option<ContractInfo>,
        proxy_band: Option<ContractInfo>,
        oracle_router: Option<ContractInfo>,
        admin_auth: Option<ContractInfo>,
    ) -> AnyResult<Self> {
        let mut core = OracleCore {
            deps: HashMap::new(),
            superadmin: admin.clone(),
        };
        let quote_symbol = "USD".to_string();
        core.superadmin = admin.clone();

        let admin_auth =
            admin_auth.unwrap_or_else(|| init_admin_auth(app, admin));

        core.deps
            .insert(OracleDeps::AdminAuth, admin_auth.clone().into());

        let band = band.unwrap_or_else(|| {
            band::InstantiateMsg {}.test_init(MockBand::default(), app, admin.clone(), "band", &[]).unwrap()
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
                app,
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
                app,
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
        .test_exec(&oracle_router, app, admin.clone(), &[])
        .unwrap();

        // Configure mock band prices
        for (sym, price) in prices {
            band::ExecuteMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: quote_symbol.clone(),
                rate: price,
                last_updated: None,
            }
            .test_exec(&band, app, admin.clone(), &[])
            .unwrap();
        }

        Ok(core)
    }

    pub fn update_prices(&self, app: &mut App, prices: HashMap<String, Uint128>, last_updated_time: u64) {
        for (sym, price) in prices {
            band::ExecuteMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: "USD".into(),
                rate: price,
                last_updated: Some(last_updated_time),
            }
            .test_exec(
                &self.get(OracleDeps::Band),
                app,
                self.superadmin.clone(),
                &[],
            )
            .unwrap();
        }
    }
}
