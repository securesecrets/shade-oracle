use shade_oracles::earn;
use shade_oracles::{
    common::types::Contract,
    ensemble::ContractEnsemble,
    ensemble_helpers::EnsembleContract,
    ensemble_new, ensemblify,
    scrt::{ContractInstantiationInfo, HumanAddr},
};
use std::{cell::RefCell, rc::Rc};

use super::common::OracleContract;

ensemblify!(
    EarnV1OracleHarness,
    ::earn_v1_oracle::contract::init,
    ::earn_v1_oracle::contract::handle,
    ::earn_v1_oracle::contract::query,
    EarnV1Oracle
);

impl OracleContract for EarnV1Oracle {}
impl EarnV1Oracle {
    ensemble_new!(EarnV1Oracle, earn::InitMsg);
}
