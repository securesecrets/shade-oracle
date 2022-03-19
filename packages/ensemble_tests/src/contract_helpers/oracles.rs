use super::EnsembleContract;
use crate::{ensemble_new, ensemblify};
use serde::{Deserialize, Serialize};
use shade_oracles::oracles::{
    band, common as common_oracles, earn_v1_oracle, lp_oracle, proxy_band_oracle, router,
};
use shared_types::{
    asset::Contract,
    composable_snip20::msg as snip20,
    ensemble::ContractEnsemble,
    scrt::{ContractInstantiationInfo, HumanAddr, StdResult, Uint128},
};
use std::{cell::RefCell, rc::Rc};

ensemblify!(
    OracleRouterHarness,
    ::oracle_router::contract::init,
    ::oracle_router::contract::handle,
    ::oracle_router::contract::query,
    OracleRouter
);

impl OracleRouter {
    pub fn query_config(&self, key: String) -> StdResult<router::ConfigResponse> {
        self.query(&router::QueryMsg::GetOwner {})
    }

    pub fn query_price(&self, key: String) -> StdResult<common_oracles::PriceResponse> {
        self.query(&router::QueryMsg::GetPrice { key })
    }

    pub fn query_oracle(&self, key: String) -> StdResult<router::OracleResponse> {
        self.query(&router::QueryMsg::GetOracle { key })
    }

    pub fn change_admin(&self, new_owner: String, sender_key: Option<&str>) -> StdResult<()> {
        let msg = router::HandleMsg::ChangeOwner { new_owner };
        self.handle(&msg, sender_key)
    }

    pub fn update_registry(
        &self,
        operation: router::RegistryOperation,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        let msg = router::HandleMsg::UpdateRegistry { operation };
        self.handle(&msg, sender_key)
    }

    pub fn batch_update_registry(
        &self,
        operations: Vec<router::RegistryOperation>,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        let msg = router::HandleMsg::BatchUpdateRegistry { operations };
        self.handle(&msg, sender_key)
    }
}

ensemblify!(
    MockBandHarness,
    mock_band::contract::init,
    mock_band::contract::handle,
    mock_band::contract::query,
    MockBand
);
ensemblify!(
    ProxyBandOracleHarness,
    ::proxy_band_oracle::contract::init,
    ::proxy_band_oracle::contract::handle,
    ::proxy_band_oracle::contract::query,
    ProxyBandOracle
);
ensemblify!(
    LpOracleHarness,
    ::lp_oracle::contract::init,
    ::lp_oracle::contract::handle,
    ::lp_oracle::contract::query,
    LpOracle
);
ensemblify!(
    EarnV1OracleHarness,
    ::earn_v1_oracle::contract::init,
    ::earn_v1_oracle::contract::handle,
    ::earn_v1_oracle::contract::query,
    EarnV1Oracle
);

pub trait OracleContract
where
    Self: EnsembleContract,
{
    fn query_price(&self) -> StdResult<common_oracles::PriceResponse> {
        self.query(&common_oracles::QueryMsg::GetPrice {})
    }
    fn query_config<Response: serde::de::DeserializeOwned + serde::Serialize>(
        &self,
    ) -> StdResult<Response> {
        self.query(&common_oracles::QueryMsg::GetConfig {})
    }
}

impl MockBand {
    pub fn new(
        ensemble: &Rc<RefCell<ContractEnsemble>>,
        init_info: &ContractInstantiationInfo,
        address: &str,
        account_key: Option<&str>,
    ) -> Self {
        let info = Self::init(
            &band::InitMsg {},
            &ensemble,
            init_info,
            address,
            account_key,
        );
        MockBand {
            info,
            ensemble: ensemble.clone(),
        }
    }

    pub fn update_band_pair(
        &self,
        pair: (&str, &str),
        rate: Uint128,
        last_updated: Option<u64>,

        account_key: Option<&str>,
    ) -> StdResult<()> {
        let msg = shade_oracles::oracles::band::HandleMsg::UpdateSymbolPrice {
            base_symbol: pair.0.to_string(),
            quote_symbol: pair.1.to_string(),
            rate: rate,
            last_updated: last_updated,
        };
        self.handle(&msg, account_key)
    }
}

impl OracleContract for ProxyBandOracle {}
impl ProxyBandOracle {
    pub fn new(
        owner: String,
        pair: (&str, &str),
        band: Contract,
        ensemble: &Rc<RefCell<ContractEnsemble>>,
        init_info: &ContractInstantiationInfo,
        address: &str,
        account_key: Option<&str>,
    ) -> Self {
        let msg = proxy_band_oracle::InitMsg {
            owner,
            base_symbol: pair.0.to_string().clone(),
            quote_symbol: pair.1.to_string().clone(),
            band,
        };
        let info = Self::init(&msg, ensemble, init_info, address, account_key);
        ProxyBandOracle {
            info,
            ensemble: ensemble.clone(),
        }
    }
}

impl OracleContract for LpOracle {}
impl LpOracle {
    ensemble_new!(LpOracle, lp_oracle::InitMsg);
}

impl OracleContract for EarnV1Oracle {}
impl EarnV1Oracle {
    ensemble_new!(EarnV1Oracle, earn_v1_oracle::InitMsg);
}

#[macro_export]
/// Create mock band with as many initial price feeds as possible (mock_band, address, ensemble, ("SCRT", "USD", Uint128(10)), ...)
macro_rules! init_band {
    ($name:ident, $addr:expr, $ensemble:expr, $($opt:expr),*) => {
        let $name = $ensemble.borrow_mut().register(Box::new(MockBandHarness));
        let $name = MockBand::new(&$ensemble, &$name, $addr, None);
        $(
            $name
            .update_band_pair($opt.0, $opt.1, None, None)
            .unwrap();
        )*
    }
}

#[macro_export]
/// Create an instance of an oracle with (type, name, address, ensemble, ...)
macro_rules! init_oracle {
    (proxy_band, $name:ident, $addr:expr, $ensemble:expr, $pair:expr, $mock_band:expr) => {
        let contract = $ensemble
        .borrow_mut()
        .register(Box::new(ProxyBandOracleHarness));
        let $name = ProxyBandOracle::new(
            DEFAULT_ADMIN.to_string(),
            $pair,
            Contract::new(&$mock_band.info),
            &$ensemble,
            &contract,
            $addr,
            None,
        );
    };
    (lp, $name:ident, $addr:expr, $ensemble:expr, $oracle0:expr, $oracle1:expr, $factory:expr, $dex:expr) => {
        let contract = $ensemble
        .borrow_mut()
        .register(Box::new(LpOracleHarness));
        let msg = lp_oracle::InitMsg {
            owner: DEFAULT_ADMIN.to_string(),
            oracle0: $oracle0.as_contract(),
            oracle1: $oracle1.as_contract(),
            factory: $factory.as_contract(),
            dex: $dex,
        };
        let $name = LpOracle::new(
            &msg,
            &$ensemble,
            &contract,
            $addr,
            None,
        );
    }
}
