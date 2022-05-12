use colored::*;
use mulberry_utils::{
    common::types::{self, Contract},
    composable_snip20::msg as snip20,
    protocols,
    scrt::ContractInstantiationInfo,
    scrt::{HumanAddr, Uint128},
};
use secretcli::cli_types::NetContract;
use secretcli::secretcli::{account_address, query_contract, test_contract_handle, test_inst_init};
use serde_json::Result;
use shade_oracles::router;
use shade_oracles::{
    band, common as common_oracles, lp::siennaswap as SiennaSwapLpOracle,
    router::RegistryOperation, staking_derivative::shade as shd_stkd,
};
use shade_oracles_integration::constants::testnet::*;
use shade_oracles_integration::constants::*;
use shade_oracles_integration::contract_helpers::oracles::ShadeStakingDerivativeOracleContract;
use shade_oracles_integration::contract_helpers::{
    oracles::{
        BandContract, EarnV1OracleContract, OracleContract, OracleRouterContract,
        ProxyBandOracleContract, SiennaswapSpotLpOracleContract,
    },
    TestableContract,
};

fn main() -> Result<()> {
    let user_a = account_address(HOOMP_KEY).unwrap_or_default();

    println!("Account A: {}", user_a.blue());

    deploy()?;
    Ok(())
}

fn deploy() -> Result<()> {
    let user_a = account_address(HOOMP_KEY).unwrap_or_default();

    println!("Account A: {}", user_a.blue());

    let band = Contract {
        address: BAND.to_string(),
        code_hash: BAND_HASH.to_string(),
    };

    let staking_derivative = Contract {
        address: STKD_SCRT.to_string(),
        code_hash: STKD_SCRT_HASH.to_string(),
    };

    let sienna_stkd_scrt_scrt_lp = Contract {
        address: STKD_SCRT_SCRT_POOL.to_string(),
        code_hash: STKD_SCRT_SCRT_POOL_HASH.to_string(),
    };

    println!("Deploying SCRT Oracle.");
    let scrt_oracle = ProxyBandOracleContract::new(
        user_a.clone(),
        ("SCRT", "USD"),
        band,
        Some(HOOMP_KEY),
        Some("scrt_oracle"),
    )?;

    println!("Deploying oracle router and configuring it.");
    let router = OracleRouterContract::new(
        &router::InitMsg {
            owner: user_a.clone(),
        },
        Some(HOOMP_KEY),
        Some("oracle_router"),
    )?;

    router.update_registry(
        RegistryOperation::Add {
            oracle: scrt_oracle.as_contract(),
            key: "SCRT".to_string(),
        },
        Some(HOOMP_KEY),
    )?;

    let scrt_rate = router.query_price("SCRT".to_string())?.rate;
    println!("Router price of SCRT: {}", scrt_rate);

    println!("Deploying stkd-SCRT oracle.");
    let stkd_scrt_oracle = ShadeStakingDerivativeOracleContract::new(
        &shd_stkd::InitMsg {
            owner: user_a.clone(),
            symbol: "SCRT".to_string(),
            staking_derivative,
            router: router.as_contract(),
        },
        Some(HOOMP_KEY),
        Some("stkd_scrt_oracle"),
    )?;

    println!("Registering stkd-SCRT oracle to router.");
    router.update_registry(
        RegistryOperation::Add {
            oracle: stkd_scrt_oracle.as_contract(),
            key: "stkd-SCRT".to_string(),
        },
        Some(HOOMP_KEY),
    )?;

    println!("Deploying stkd-SCRT / SCRT Siennaswap LP oracle.");
    let stkd_scrt_scrt_lp_oracle = SiennaswapSpotLpOracleContract::new(
        &SiennaSwapLpOracle::InitMsg {
            owner: user_a,
            symbol_0: "stkd-SCRT".to_string(),
            symbol_1: "SCRT".to_string(),
            router: router.as_contract(),
            factory: sienna_stkd_scrt_scrt_lp,
        },
        Some(HOOMP_KEY),
        Some("stkd_scrt_scrt_lp_oracle"),
    )?;

    println!("Registering stkd-SCRT/SCRT oracle to router.");
    router.update_registry(
        RegistryOperation::Replace {
            oracle: stkd_scrt_scrt_lp_oracle.as_contract(),
            key: "stkd-SCRT/SCRT SiennaSwap LP".to_string(),
        },
        Some(HOOMP_KEY),
    )?;

    Ok(())
}
