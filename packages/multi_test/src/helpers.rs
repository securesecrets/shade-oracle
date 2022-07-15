use cosmwasm_std::{Addr, ContractInfo, StdResult};
use shade_oracles::{
    Contract,
    interfaces::{
    band::{self, proxy},
    common::Contract,
    router,
    }
};
use shade_protocol::{multi_test::App, utils::{InstantiateCallback, ExecuteCallback, Query, MultiTestable}};
use crate::multi::{AdminAuth, MockBand, OracleRouter, ProxyBandOracle};

pub struct OracleCore {
    pub band: ContractInfo,
    pub band_proxy: ContractInfo,
    pub router: ContractInfo,
    pub admin_auth: ContractInfo,
}

pub fn setup_core(&mut app: App) -> StdResult<OracleCore> {
    let admin = "admin";

    let admin_auth =shade_admin::admin::InstantiateMsg { super_admin: None }.test_init(AdminAuth::default(), router, sender, label, send_funds)?;


    let band = band::InstantiateMsg {}.test_init(MockBand::default(), app, admin, "band", &vec![])?;

    let band_proxy = proxy::InstantiateMsg {
                band: band.into(),
                quote_symbol: "USD".to_string(),
                admin_auth: Contract::new_link(admin_auth.clone()),
            }.test_init(ProxyBandOracle::default(), app, admin, "proxy-band", &vec![])?;

    let router = router::InstantiateMsg {
        default_oracle: band_proxy.into(),
        admin_auth: admin_auth.into(),
        band: band.into(),
        quote_symbol: "USD".to_string(),
    }.test_init(OracleRouter::default(), app, admin, "oracle-router", &vec![])?;

    shade_admin::admin::ExecuteMsg::AddContract {
        contract_address: router.address.to_string(),
    }

    ensemble
        .execute(
            &shade_admin::admin::ExecuteMsg::AddAuthorization {
                contract_address: router.address.to_string(),
                admin_address: "admin".to_string(),
            },
            MockEnv::new("admin", admin_auth.clone()),
        )
        .unwrap();

    ensemble
        .execute(
            &shade_admin::admin::ExecuteMsg::AddContract {
                contract_address: band_proxy.address.to_string(),
            },
            MockEnv::new("admin", admin_auth.clone()),
        )
        .unwrap();

    ensemble
        .execute(
            &shade_admin::admin::ExecuteMsg::AddAuthorization {
                contract_address: band_proxy.address.to_string(),
                admin_address: "admin".to_string(),
            },
            MockEnv::new("admin", admin_auth.clone()),
        )
        .unwrap();

    OracleEnsembleCore {
        band,
        band_proxy,
        router,
        admin_auth,
        ensemble,
    }
}
