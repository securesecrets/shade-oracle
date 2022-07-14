use cosmwasm_std::Addr;
use shade_ensemble::{
    core::ContractLink,
    ensemble::{ContractEnsemble, MockEnv},
};
use shade_oracles::{
    band::{self, proxy},
    common::Contract,
    router,
};

use crate::harness::{AdminAuth, MockBand, OracleRouter, ProxyBandOracle};

pub struct OracleEnsembleCore {
    pub band: Contract,
    pub band_proxy: Contract,
    pub router: Contract,
    pub admin_auth: Contract,
    pub ensemble: ContractEnsemble,
}

pub fn setup_core(mut ensemble: ContractEnsemble) -> OracleEnsembleCore {
    let reg_router = ensemble.register(Box::new(OracleRouter));
    let reg_mock_band = ensemble.register(Box::new(MockBand));
    let reg_mock_band_proxy = ensemble.register(Box::new(ProxyBandOracle));
    let reg_admin_auth = ensemble.register(Box::new(AdminAuth));

    let admin_auth = ensemble
        .instantiate(
            reg_admin_auth.id,
            &shade_admin::admin::InstantiateMsg {},
            MockEnv::new(
                "admin",
                ContractLink {
                    address: Addr("admin_auth".into()),
                    code_hash: reg_admin_auth.code_hash,
                },
            ),
        )
        .unwrap()
        .instance;

    let band = ensemble
        .instantiate(
            reg_mock_band.id,
            &band::InstantiateMsg {},
            MockEnv::new(
                "admin",
                ContractLink {
                    address: Addr("band".into()),
                    code_hash: reg_mock_band.code_hash.clone(),
                },
            ),
        )
        .unwrap()
        .instance;

    let band_proxy = ensemble
        .instantiate(
            reg_mock_band_proxy.id,
            &proxy::InstantiateMsg {
                band: Contract {
                    address: band.address.clone(),
                    code_hash: band.code_hash.clone(),
                },
                quote_symbol: "USD".to_string(),
                admin_auth: Contract::new_link(admin_auth.clone()),
            },
            MockEnv::new(
                "admin",
                ContractLink {
                    address: Addr("band_proxy".into()),
                    code_hash: reg_mock_band.code_hash,
                },
            ),
        )
        .unwrap()
        .instance;

    let router = ensemble
        .instantiate(
            reg_router.id,
            &router::InstantiateMsg {
                default_oracle: Contract {
                    address: band_proxy.address.clone(),
                    code_hash: band_proxy.code_hash.clone(),
                },
                admin_auth: Contract::new_link(admin_auth.clone()),
                band: Contract::new(band.address.to_string(), band.code_hash.clone()),
                quote_symbol: "USD".to_string(),
            },
            MockEnv::new(
                "admin",
                ContractLink {
                    address: Addr("router".into()),
                    code_hash: reg_router.code_hash,
                },
            ),
        )
        .unwrap()
        .instance;

    ensemble
        .execute(
            &shade_admin::admin::ExecuteMsg::AddContract {
                contract_address: router.address.to_string(),
            },
            MockEnv::new("admin", admin_auth.clone()),
        )
        .unwrap();

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
