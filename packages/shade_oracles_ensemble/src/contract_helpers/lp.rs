use shade_oracles::{
    lp
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
    LpOracleHarness,
    ::lp_oracle::contract::init,
    ::lp_oracle::contract::handle,
    ::lp_oracle::contract::query,
    LpOracle
);

impl OracleContract for LpOracle {}
impl LpOracle {
    ensemble_new!(LpOracle, lp::InitMsg);
}