use colored::*;
use secretcli::cli_types::NetContract;
use secretcli::secretcli::{account_address, query_contract, test_contract_handle, test_inst_init};
use serde_json::Result;
use shade_oracles::{router, siennaswap_market_oracle};
use shade_oracles::{
    band, common as common_oracles, lp::siennaswap as SiennaSwapLpOracle,
    router::RegistryOperation, staking_derivative::shade as shd_stkd,
    index_oracle,
};
use shade_oracles::{
    common::{self, Contract},
    protocols,
};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{HumanAddr};
use shade_oracles_integration::constants::testnet::*;
use shade_oracles_integration::constants::*;
use shade_oracles_integration::contract_helpers::{
    oracles::{
        BandContract, EarnV1OracleContract, OracleContract, OracleRouterContract,
        ProxyBandOracleContract, SiennaswapSpotLpOracleContract,
        ShadeStakingDerivativeOracleContract, IndexOracleContract,
        SiennaMarketOracleContract,
    },
    TestableContract,
};

fn main() -> Result<()> {
    let user_a = account_address(HOOMP_KEY).unwrap_or_default();
    let band = Contract::new(BAND.to_string(), BAND_HASH.to_string());

    println!("Account A: {}", user_a.blue());


    let oracle_router = deploy_router(user_a.clone(), band.clone())?;
    let shd_oracle = deploy_shd(user_a.clone(), oracle_router.as_contract())?;
    let silk_oracle = deploy_silk(user_a.clone(), oracle_router.as_contract())?;
    let stkd_scrt_oracle = deploy_stkd_scrt(user_a.clone(), oracle_router.as_contract())?;
    let stkd_scrt_scrt_lp_oracle = deploy_stkd_scrt_scrt_lp(user_a, oracle_router.as_contract())?;
    
    oracle_router.update_oracle(HOOMP_KEY, "SILK", silk_oracle.as_contract())?;
    oracle_router.update_oracle(HOOMP_KEY, "SHD", shd_oracle.as_contract())?;
    oracle_router.update_oracle(HOOMP_KEY, "stkd-SCRT", stkd_scrt_oracle.as_contract())?;
    oracle_router.update_oracle(HOOMP_KEY, "stkd-SCRT/SCRT SiennaSwap LP", stkd_scrt_scrt_lp_oracle.as_contract())?;
    oracle_router.update_registry(RegistryOperation::UpdateAlias { alias: "SSCRT".to_string(), key: "SCRT".to_string() }, Some(HOOMP_KEY))?;
    let shd_price = oracle_router.query_price("SHD".to_string());
    match shd_price {
        Ok(price) =>     println!("SHD Price is: {}", price.data.rate),
        Err(err) => println!("{}", err),
    }
    Ok(())
}

fn deploy_silk(user_a: String, router: Contract) -> Result<IndexOracleContract> {
    println!("Deploying SILK oracle.");

    let silk_oracle = IndexOracleContract::new(
        &index_oracle::InitMsg {
            admins: None,
            router,
            symbol: "SILK".to_string(),
            basket: vec![
                ("USD", 39_33 * 10u128.pow(14)), //  39.32%
                ("CNY", 7_13 * 10u128.pow(14)), //  7.13%
                ("EUR", 15_97 * 10u128.pow(14)), // 15.97%
                ("JPY", 7_64 * 10u128.pow(14)), //  7.64%
                ("GBP", 3_40 * 10u128.pow(14)), //  3.4%
                ("CAD", 4_58 * 10u128.pow(14)), //  4.58%
                ("KRW", 1_53 * 10u128.pow(14)), //  1.53%
                ("AUD", 2_32 * 10u128.pow(14)), //  2.32%
                ("IDR", 2_50 * 10u128.pow(14)), //  2.5%
                ("CHF", 4_44 * 10u128.pow(14)), //  4.44%
                ("SEK", 0_84 * 10u128.pow(14)), //  0.84%
                ("NOK", 0_82 * 10u128.pow(14)), //  0.82%
                ("SGD", 2_50 * 10u128.pow(14)), //  2.5%
                ("XAU", 5_00 * 10u128.pow(14)), //  5.0%
                ("WBTC", 2_00 * 10u128.pow(14)), //  2.0%
            ].into_iter().map(|(sym, w)| (sym.to_string(), Uint128::from(w))).collect(),
            target: Uint128::from(1_05 * 10u128.pow(16)), // $1.05
        },
        Some(HOOMP_KEY),
        Some("silk-oracle"),
    )?;

    Ok(silk_oracle)
}

fn deploy_router(user_a: String, band: Contract) -> Result<OracleRouterContract> {

    println!("Deploying Band Oracle.");
    let scrt_oracle = ProxyBandOracleContract::new(
        user_a.clone(),
        "USD",
        band.clone(),
        Some(HOOMP_KEY),
        Some("band_oracle"),
    )?;

    println!("Deploying oracle router.");
    let router = OracleRouterContract::new(
        &router::InitMsg {
            owner: HumanAddr(user_a.clone()),
            default_oracle: scrt_oracle.as_contract(),
        },
        Some(HOOMP_KEY),
        Some("oracle_router"),
    )?;

    Ok(router)
}

fn deploy_shd(user_a: String, router: Contract) -> Result<(SiennaMarketOracleContract)> {

    println!("Deploying SHD oracle looking at Sienna SHD/SSCRT base pegged to SCRT.");
    let sienna_scrt_shd = Contract::new(SIENNA_SHD_SSCRT_PAIR.to_string(), SIENNA_SHD_SSCRT_PAIR_HASH.to_string());
    let shd_oracle = SiennaMarketOracleContract::new(
        &siennaswap_market_oracle::InitMsg {
            admins: None,
            router,
            pair: sienna_scrt_shd,
            symbol: "SHD".into(),
            base_peg: Some("SCRT".to_string()),
        },
        Some(HOOMP_KEY),
        Some("sienna-market-shd-oracle"),
    )?;
    Ok(shd_oracle)  
}

fn deploy_stkd_scrt(user_a: String, router: Contract) -> Result<ShadeStakingDerivativeOracleContract> {
    let staking_derivative = Contract::new(STKD_SCRT.to_string(), STKD_SCRT_HASH.to_string());

    println!("Deploying stkd-SCRT oracle.");
    let stkd_scrt_oracle = ShadeStakingDerivativeOracleContract::new(
        &shd_stkd::InitMsg {
            owner: HumanAddr(user_a.clone()),
            supported_key: "stkd-SCRT".to_string(),
            underlying_symbol: "SCRT".to_string(),
            staking_derivative,
            router,
        },
        Some(HOOMP_KEY),
        Some("stkd_scrt_oracle"),
    )?;

    Ok(stkd_scrt_oracle)
}

fn deploy_stkd_scrt_scrt_lp(user_a: String, router: Contract) -> Result<SiennaswapSpotLpOracleContract> {

    let sienna_stkd_scrt_scrt_lp = Contract::new(STKD_SCRT_SCRT_POOL.to_string(), STKD_SCRT_SCRT_POOL_HASH.to_string());

    println!("Deploying stkd-SCRT / SCRT Siennaswap LP oracle.");
    let stkd_scrt_scrt_lp_oracle = SiennaswapSpotLpOracleContract::new(
        &SiennaSwapLpOracle::InitMsg {
            owner: HumanAddr(user_a),
            symbol_0: "stkd-SCRT".to_string(),
            symbol_1: "SCRT".to_string(),
            router,
            exchange: sienna_stkd_scrt_scrt_lp,
            supported_key: "stkd-SCRT/SCRT SiennaSwap LP".to_string(),
        },
        Some(HOOMP_KEY),
        Some("stkd_scrt_scrt_lp_oracle"),
    )?;

    Ok(stkd_scrt_scrt_lp_oracle)
}
