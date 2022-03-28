use shade_oracles::{
    band::{proxy}
};
use mulberry_utils::{
    ensemblify,
    ensemble_helpers::EnsembleContract,
    common::{types::Contract},
    ensemble::ContractEnsemble,
    scrt::{ContractInstantiationInfo, HumanAddr},
};
use std::{cell::RefCell, rc::Rc};

use super::common::OracleContract;

ensemblify!(
    ProxyBandOracleHarness,
    ::proxy_band_oracle::contract::init,
    ::proxy_band_oracle::contract::handle,
    ::proxy_band_oracle::contract::query,
    ProxyBandOracle
);

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
        let msg = proxy::InitMsg {
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