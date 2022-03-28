use shade_oracles::{
    earn
};
use mulberry_utils::{
    ensemble_new, ensemblify,
    ensemble_helpers::EnsembleContract,
    common::{types::Contract},
    ensemble::ContractEnsemble,
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