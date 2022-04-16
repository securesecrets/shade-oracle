use mulberry_utils::{ensemble_helpers::EnsembleContract, scrt::StdResult};
use shade_oracles::common as common_oracles;

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

#[macro_export]
/// Create an instance of an oracle with (type, name, address, ensemble, ...)
macro_rules! init_oracle {
    (proxy_band, $name:ident, $addr:expr, $ensemble:expr, $pair:expr, $mock_band:expr) => {
        let contract = $ensemble.borrow_mut().register(Box::new(
            shade_oracles_ensemble::contract_helpers::proxy_band::ProxyBandOracleHarness,
        ));
        let $name = crate::contract_helpers::proxy_band::ProxyBandOracle::new(
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
        let contract = $ensemble.borrow_mut().register(Box::new(
            shade_oracles_ensemble::contract_helpers::lp::LpOracleHarness,
        ));
        let msg = shade_oracles_ensemble::contract_helpers::lp::lp_oracle::InitMsg {
            owner: DEFAULT_ADMIN.to_string(),
            oracle0: $oracle0.as_contract(),
            oracle1: $oracle1.as_contract(),
            factory: $factory.as_contract(),
            dex: $dex,
        };
        let $name = LpOracle::new(&msg, &$ensemble, &contract, $addr, None);
    };
}
