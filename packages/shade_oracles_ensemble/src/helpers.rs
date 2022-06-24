use cosmwasm_std::HumanAddr;
use fadroma::{ensemble::{self, ContractEnsemble}, core::ContractLink};
use shade_oracles::common::Contract;

pub struct OracleEnsembleCore {
    pub band: ContractLink<HumanAddr>,
    pub band_proxy: ContractLink<HumanAddr>,
    pub router: ContractLink<HumanAddr>,
    pub admin_auth: ContractLink<HumanAddr>,
}

pub fn setup_core(ensemble: ContractEnsemble) -> OracleEnsembleCore {
    let reg_router = ensemble.register(Box::new(OracleRouter));
    let reg_mock_band = ensemble.register(Box::new(MockBand));
    let reg_mock_band_proxy = ensemble.register(Box::new(ProxyBandOracle));
    let reg_admin_auth = ensemble.register(Box::new(AdminAuth));

    let admin_auth = ensemble.instantiate(
        reg_admin_auth.id,
        &shade_admin::admin::InitMsg { },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("admin_auth".into()),
                code_hash: reg_admin_auth.code_hash.clone(),
            }
        )
    ).unwrap().instance;
    
    let band = ensemble.instantiate(
        reg_mock_band.id,
        &band::InitMsg { },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("band".into()),
                code_hash: reg_mock_band.code_hash.clone(),
            }
        )
    ).unwrap().instance;

    let band_proxy = ensemble.instantiate(
        reg_mock_band_proxy.id,
        &proxy::InitMsg {
            band: Contract {
                address: band.address.clone(),
                code_hash: band.code_hash.clone(),
            },
            quote_symbol: "USD".to_string(),
            router: Contract::new_link(admin_auth),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("band_proxy".into()),
                code_hash: reg_mock_band.code_hash.clone(),
            }
        )
    ).unwrap().instance;

    let router = ensemble.instantiate(
        reg_router.id,
        &router::InitMsg {
            default_oracle: Contract {
                address: band_proxy.address.clone(),
                code_hash: band_proxy.code_hash.clone(),
            },
            admin_auth: todo!(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("router".into()),
                code_hash: reg_router.code_hash.clone(),
            }
        )
    ).unwrap().instance;

    ensemble.execute(
        &shade_admin::admin::HandleMsg::AddContract {
            contract_address: router.address.clone(),
        },
        MockEnv::new(
            "admin", 
            admin_auth.clone(),
        ),
    ).unwrap();

    ensemble.execute(
        &shade_admin::admin::HandleMsg::AddAuthorization { contract_address: router.address.clone(), admin_address: "admin".to_string() },
        MockEnv::new(
            "admin",
            admin_auth.clone(),
        )
    ).unwrap();

    OracleEnsembleCore {
        band,
        band_proxy,
        router,
        admin_auth,
    }
}