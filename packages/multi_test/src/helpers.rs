use std::collections::HashMap;

use shade_oracles::{
    interfaces::{
    band::{self, proxy},
    router::{self, UpdateConfig},
    }, common::{InstantiateCommonConfig}
};
use shade_admin_multi_test::multi::AdminAuth;
//use shade_multi_test::multi::snip20::Snip20;
use shade_protocol::{
    AnyResult,
    c_std::{Addr, ContractInfo, StdResult, Uint128},
    Contract,
    multi_test::App, 
    utils::{InstantiateCallback, ExecuteCallback, MultiTestable}
};
use crate::multi::{MockBand, OracleRouter, ProxyBandOracle};

pub struct OracleCore {
    pub band: ContractInfo,
    pub band_proxy: ContractInfo,
    pub router: ContractInfo,
    pub admin_auth: ContractInfo,
    pub admin: Addr,
}

impl OracleCore {
/// Initializes the core dependencies for testing all oracles which are
/// band, proxy band, router, and the admin auth contract. Then, it updates the prices in band
/// based off the prices argument with them being quoted in "USD".
pub fn setup(app: &mut App, prices: HashMap<String, Uint128>) -> AnyResult<OracleCore> {
    let admin = Addr::unchecked("superadmin");

    let admin_auth =shade_admin::admin::InstantiateMsg { super_admin: None }.test_init(AdminAuth::default(), app, admin.clone(), "admin-auth", &[])?;

    let band = band::InstantiateMsg {}.test_init(MockBand::default(), app, admin.clone(), "band", &[])?;

    let router = router::InstantiateMsg {
        default_oracle: admin_auth.clone().into(),
        admin_auth: admin_auth.clone().into(),
        band: band.clone().into(),
        quote_symbol: "USD".to_string(),
    }.test_init(OracleRouter::default(), app, admin.clone(), "oracle-router", &[])?;

    let band_proxy = proxy::InstantiateMsg {
        quote_symbol: "USD".to_string(),
        config: InstantiateCommonConfig::new(None, router.clone().into(), true, true),
        band: band.clone().into(),
    }.test_init(ProxyBandOracle::default(), app, admin.clone(), "proxy-band", &[])?;

    let mut operations = vec![];

    for (sym, _) in prices.clone() {
        operations.push(router::RegistryOperation::Add {
            oracle: Contract {
                address: band_proxy.address.clone(),
                code_hash: band_proxy.code_hash.clone(),
            },
            key: sym,
        });
    }

    // Configure BAND symbols on router
    router::ExecuteMsg::BatchUpdateRegistry { operations }.test_exec(&router.clone(), app, admin.clone(), &[]).unwrap();

    router::ExecuteMsg::UpdateConfig { config: UpdateConfig {
        admin_auth: None,
        default_oracle: Some(band_proxy.clone().into()),
        band: None,
        quote_symbol: None,
        enabled: None,
    } }.test_exec(&router, app, admin.clone(), &[]).unwrap();
    
    // Configure mock band prices
    for (sym, price) in prices {
        band::ExecuteMsg::UpdateSymbolPrice {
                    base_symbol: sym,
                    quote_symbol: "USD".to_string(),
                    rate: price,
                    last_updated: None,
        }.test_exec(&band, app, admin.clone(), &[]).unwrap();
    }

    Ok(OracleCore {
        band,
        band_proxy,
        router,
        admin_auth,
        admin,
    })
}

pub fn update_prices(&self, app: &mut App, prices: HashMap<String, Uint128>, last_updated_time: u64) {
    for (sym, price) in prices {
        band::ExecuteMsg::UpdateSymbolPrice {
                    base_symbol: sym,
                    quote_symbol: "USD".to_string(),
                    rate: price,
                    last_updated: Some(last_updated_time),
        }.test_exec(&self.band, app, self.admin.clone(), &[]).unwrap();
    }
}
}