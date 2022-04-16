use shade_oracles::{
    band::{self}
};
use mulberry_utils::{
    ensemblify,
    ensemble_helpers::EnsembleContract,
    common::{types::Contract},
    ensemble::ContractEnsemble,
    scrt::{ContractInstantiationInfo, HumanAddr, StdResult, Uint128},
};
use std::{cell::RefCell, rc::Rc};

ensemblify!(
    MockBandHarness,
    mock_band::contract::init,
    mock_band::contract::handle,
    mock_band::contract::query,
    MockBand
);

impl MockBand {
    pub fn new(
        ensemble: &Rc<RefCell<ContractEnsemble>>,
        init_info: &ContractInstantiationInfo,
        address: &str,
        account_key: Option<&str>,
    ) -> Self {
        let info = Self::init(
            &band::InitMsg {},
            ensemble,
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
        let msg = shade_oracles::band::HandleMsg::UpdateSymbolPrice {
            base_symbol: pair.0.to_string(),
            quote_symbol: pair.1.to_string(),
            rate,
            last_updated,
        };
        self.handle(&msg, account_key)
    }
}

#[macro_export]
/// Create mock band with as many initial price feeds as possible (mock_band, address, ensemble, ("SCRT", "USD", Uint128(10)), ...)
macro_rules! init_band {
    ($name:ident, $addr:expr, $ensemble:expr, $($opt:expr),*) => {
        let $name = $ensemble.borrow_mut().register(Box::new(shade_oracles_ensemble::contract_helpers::mock_band::MockBandHarness));
        let $name = shade_oracles_ensemble::contract_helpers::mock_band::MockBand::new(&$ensemble, &$name, $addr, None);
        $(
            $name
            .update_band_pair($opt.0, $opt.1, None, None)
            .unwrap();
        )*
    }
}
