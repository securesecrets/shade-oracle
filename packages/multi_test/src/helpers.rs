use cosmwasm_std::{Addr, ContractInfo, StdResult};
use shade_oracles::{
    Contract,
    interfaces::{
    band::{self, proxy},
    router,
    }
};
use shade_admin_multi_test::AdminAuth;
use shade_protocol::{multi_test::App, utils::{InstantiateCallback, ExecuteCallback, Query, MultiTestable}};
use crate::multi::{MockBand, OracleRouter, ProxyBandOracle};

pub struct OracleCore {
    pub band: ContractInfo,
    pub band_proxy: ContractInfo,
    pub router: ContractInfo,
    pub admin_auth: ContractInfo,
}

pub fn setup_core(app: &mut App) -> StdResult<OracleCore> {
    let admin = Addr::unchecked("superadmin");

    let admin_auth =shade_admin::admin::InstantiateMsg { super_admin: None }.test_init(AdminAuth::default(), app, admin.clone(), "admin-auth", &[])?;


    let band = band::InstantiateMsg {}.test_init(MockBand::default(), app, admin.clone(), "band", &[])?;

    let band_proxy = proxy::InstantiateMsg {
                band: band.clone().into(),
                quote_symbol: "USD".to_string(),
                admin_auth: admin_auth.clone().into(),
            }.test_init(ProxyBandOracle::default(), app, admin.clone(), "proxy-band", &[])?;

    let router = router::InstantiateMsg {
        default_oracle: band_proxy.clone().into(),
        admin_auth: admin_auth.clone().into(),
        band: band.clone().into(),
        quote_symbol: "USD".to_string(),
    }.test_init(OracleRouter::default(), app, admin.clone(), "oracle-router", &[])?;

    Ok(OracleCore {
        band,
        band_proxy,
        router,
        admin_auth,
    })
}
