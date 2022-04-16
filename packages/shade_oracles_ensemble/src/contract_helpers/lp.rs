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

pub mod secretswap {
    use super::*;
    ensemblify!(
        SecretswapLpOracleHarness,
        ::secretswap_lp_oracle::contract::init,
        ::secretswap_lp_oracle::contract::handle,
        ::secretswap_lp_oracle::contract::query,
        SecretswapLpOracle
    );
    
    impl OracleContract for SecretswapLpOracle {}
    impl SecretswapLpOracle {
        ensemble_new!(SecretswapLpOracle, lp::secretswap::InitMsg);
    }
}

pub mod siennaswap {
    use super::*;

    ensemblify!(
        SiennaswapLpOracleHarness,
        ::siennaswap_lp_oracle::contract::init,
        ::siennaswap_lp_oracle::contract::handle,
        ::siennaswap_lp_oracle::contract::query,
        SiennaswapLpOracle
    );
    
    impl OracleContract for SiennaswapLpOracle {}
    impl SiennaswapLpOracle {
        ensemble_new!(SiennaswapLpOracle, lp::siennaswap::InitMsg);
    }
}
