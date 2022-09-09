use std::fs::File;
use std::io;
use std::path::Path;

use colored::*;
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::HumanAddr;
use secretcli::cli_types::NetContract;
use secretcli::secretcli::account_address;
use serde::Serialize;
use serde_json::Result;
use shade_oracles::common::Contract;
use shade_oracles::siennaswap_market_oracle;
use shade_oracles::{
    band, index_oracle, lp::siennaswap as SiennaSwapLpOracle, router, router::RegistryOperation,
    staking_derivative::shade as shd_stkd,
};
use shade_oracles_integration::constants::*;
use shade_oracles_integration::constants::{keys, testnet::*};
use shade_oracles_integration::contract_helpers::{
    oracles::{
        IndexOracleContract, OracleRouterContract, ProxyBandOracleContract,
        ShadeStakingDerivativeOracleContract, SiennaMarketOracleContract,
        SiennaswapSpotLpOracleContract,
    },
    TestableContract,
};

const JSON_PATH: &str = "../../../jsons/";

pub fn save_json(name: &str, msg: &impl Serialize) -> io::Result<()> {
    let path = format!("{}{}.json", JSON_PATH, name);
    println!("{}", path);
    std::fs::write(path, &serde_json::to_string_pretty(&msg).unwrap())
}

fn main() -> Result<()> {
    let user_a = account_address(HOOMP_KEY).unwrap_or_default();
    let band = Contract::new(BAND.to_string(), BAND_HASH.to_string());
    let admin_auth = Contract::new(ADMIN_AUTH.to_string(), ADMIN_AUTH_HASH.to_string());

    println!("Account A: {}", user_a.blue());

    let proxy_band_oracle = deploy_proxy(admin_auth.clone(), band.clone())?;

    let msg = router::HandleMsg::UpdateRegistry {
        operation: RegistryOperation::Add {
            oracle: Contract {
                address: HumanAddr::from("test"),
                code_hash: "test".to_string(),
            },
            key: "scrt".to_string(),
        },
    };

    save_json("sample-msg", &msg).unwrap();

    let router = deploy_router(
        user_a.clone(),
        proxy_band_oracle.as_contract(),
        band.clone(),
        admin_auth.clone(),
    )?;

    // let oracle_router = OracleRouterContract {
    //     info: NetContract {
    //         label: "IMTe1EHM".to_string(),
    //         id: "10023".to_owned(),
    //         address: "secret1p4gt083fnx0uwum0q4qng08rwpddfm2jummph2".to_string(),
    //         code_hash: "8416B5F0E7B3E88886EA4C769349980A9C95459076621A7D28344DFE35F6ADEF"
    //             .to_string(),
    //     },
    // };
    //
    // // let shd_oracle = deploy_shd(user_a.clone(), oracle_router.as_contract())?;
    // // let silk_oracle = deploy_silk(user_a.clone(), oracle_router.as_contract())?;
    // // let stkd_scrt_oracle = deploy_stkd_scrt(user_a.clone(), oracle_router.as_contract())?;
    // // let stkd_scrt_scrt_lp_oracle = deploy_stkd_scrt_scrt_lp(user_a, oracle_router.as_contract())?;
    //
    // let shd_scrt_lp_oracle = deploy_sscrt_shd_lp(user_a.clone(), oracle_router.as_contract())?;
    // let stkd_scrt_shd_lp_oracle =
    //     deploy_stkd_scrt_shd_lp(user_a.clone(), oracle_router.as_contract())?;
    // // oracle_router.update_oracle(HOOMP_KEY, "SILK", silk_oracle.as_contract())?;
    // // oracle_router.update_oracle(HOOMP_KEY, "SHD", shd_oracle.as_contract())?;
    // // oracle_router.update_oracle(HOOMP_KEY, "stkd-SCRT", stkd_scrt_oracle.as_contract())?;
    // //oracle_router.update_oracle(HOOMP_KEY, "stkd-SCRT/SCRT SiennaSwap LP", stkd_scrt_scrt_lp_oracle.as_contract())?;
    // oracle_router.update_oracle(
    //     HOOMP_KEY,
    //     "SHD/SSCRT SiennaSwap LP",
    //     shd_scrt_lp_oracle.as_contract(),
    // )?;
    // oracle_router.update_oracle(
    //     HOOMP_KEY,
    //     STKD_SCRT_SHD_LP_TOKEN_NAME,
    //     stkd_scrt_shd_lp_oracle.as_contract(),
    // );
    // oracle_router.update_registry(
    //     RegistryOperation::UpdateAlias {
    //         alias: STKD_SCRT_SHD_LP_KEY.to_string(),
    //         key: STKD_SCRT_SHD_LP_TOKEN_NAME.to_string(),
    //     },
    //     Some(HOOMP_KEY),
    // )?;
    // oracle_router.update_registry(
    //     RegistryOperation::UpdateAlias {
    //         alias: "SSCRT".to_string(),
    //         key: "SCRT".to_string(),
    //     },
    //     Some(HOOMP_KEY),
    // )?;
    // let price = oracle_router.query_price("SHD/SSCRT SiennaSwap LP".to_string());
    // match price {
    //     Ok(price) => println!("SHD/sSCRT Price is: {}", price.data.rate),
    //     Err(err) => println!("{}", err),
    // }
    Ok(())
}

#[allow(clippy::mistyped_literal_suffixes, clippy::zero_prefixed_literal)]
fn deploy_silk(_user_a: String, router: Contract) -> Result<IndexOracleContract> {
    println!("Deploying SILK oracle.");

    let silk_oracle = IndexOracleContract::new(
        &index_oracle::InitMsg {
            router,
            symbol: "SILK".to_string(),
            basket: vec![
                ("USD", 39_33 * 10u128.pow(14)), //  39.32%
                ("CNY", 7_13 * 10u128.pow(14)),  //  7.13%
                ("EUR", 15_97 * 10u128.pow(14)), // 15.97%
                ("JPY", 7_64 * 10u128.pow(14)),  //  7.64%
                ("GBP", 3_40 * 10u128.pow(14)),  //  3.4%
                ("CAD", 4_58 * 10u128.pow(14)),  //  4.58%
                ("KRW", 1_53 * 10u128.pow(14)),  //  1.53%
                ("AUD", 2_32 * 10u128.pow(14)),  //  2.32%
                ("IDR", 2_50 * 10u128.pow(14)),  //  2.5%
                ("CHF", 4_44 * 10u128.pow(14)),  //  4.44%
                ("SEK", 0_84 * 10u128.pow(14)),  //  0.84%
                ("NOK", 0_82 * 10u128.pow(14)),  //  0.82%
                ("SGD", 2_50 * 10u128.pow(14)),  //  2.5%
                ("XAU", 5_00 * 10u128.pow(14)),  //  5.0%
                ("WBTC", 2_00 * 10u128.pow(14)), //  2.0%
            ]
            .into_iter()
            .map(|(sym, w)| (sym.to_string(), Uint128::from(w)))
            .collect(),
            target: Uint128::from(1_05 * 10u128.pow(16)), // $1.05
            only_band: true,
        },
        Some(HOOMP_KEY),
        Some("silk-oracle"),
        "silk-oracle",
    )?;

    Ok(silk_oracle)
}

fn deploy_proxy(admin_auth: Contract, band: Contract) -> Result<ProxyBandOracleContract> {
    ProxyBandOracleContract::new(
        admin_auth.clone(),
        "USD",
        band.clone(),
        Some(HOOMP_KEY),
        Some("proxy_band_oracle"),
        "proxy-band-usd-denom-oracle-v0.1",
    )
}

fn deploy_router(
    _user_a: String,
    proxy_band: Contract,
    band: Contract,
    admin_auth: Contract,
) -> Result<OracleRouterContract> {
    println!("Deploying oracle router.");
    let router = OracleRouterContract::new(
        &router::InitMsg {
            admin_auth,
            default_oracle: proxy_band,
            band,
            quote_symbol: "USD".to_string(),
        },
        Some(HOOMP_KEY),
        Some("oracle_router"),
        "oracle-router-v0.1",
    )?;

    Ok(router)
}

fn deploy_shd(user_a: String, router: Contract) -> Result<(SiennaMarketOracleContract)> {
    println!("Deploying SHD oracle looking at Sienna SHD/SSCRT base pegged to SCRT.");
    let sienna_scrt_shd = Contract::new(
        SIENNA_SHD_SSCRT_POOL.to_string(),
        SIENNA_SHD_SSCRT_POOL_HASH.to_string(),
    );
    let shd_oracle = SiennaMarketOracleContract::new(
        &siennaswap_market_oracle::InitMsg {
            router,
            pair: sienna_scrt_shd,
            symbol: "SHD".into(),
            base_peg: Some("SCRT".to_string()),
            only_band: true,
        },
        Some(HOOMP_KEY),
        Some("sienna-market-shd-oracle"),
        "shd-market-oracle-v0.1",
    )?;
    Ok(shd_oracle)
}

fn deploy_stkd_scrt(
    user_a: String,
    router: Contract,
) -> Result<ShadeStakingDerivativeOracleContract> {
    let staking_derivative = Contract::new(STKD_SCRT.to_string(), STKD_SCRT_HASH.to_string());

    println!("Deploying stkd-SCRT oracle.");
    let stkd_scrt_oracle = ShadeStakingDerivativeOracleContract::new(
        &shd_stkd::InitMsg {
            supported_key: "stkd-SCRT".to_string(),
            underlying_symbol: "SCRT".to_string(),
            staking_derivative,
            router,
        },
        Some(HOOMP_KEY),
        Some("stkd_scrt_oracle"),
        "shade-stkd-scrt-oracle-v0.1",
    )?;

    Ok(stkd_scrt_oracle)
}

fn deploy_stkd_scrt_scrt_lp(
    user_a: String,
    router: Contract,
) -> Result<SiennaswapSpotLpOracleContract> {
    let sienna_stkd_scrt_scrt_lp = Contract::new(
        SIENNA_STKD_SCRT_SCRT_POOL.to_string(),
        SIENNA_STKD_SCRT_SCRT_POOL_HASH.to_string(),
    );

    println!("Deploying stkd-SCRT/SCRT Siennaswap LP oracle.");
    let stkd_scrt_scrt_lp_oracle = SiennaswapSpotLpOracleContract::new(
        &SiennaSwapLpOracle::InitMsg {
            symbol_0: "stkd-SCRT".to_string(),
            symbol_1: "SCRT".to_string(),
            router,
            exchange: sienna_stkd_scrt_scrt_lp,
            supported_key: "stkd-SCRT/SCRT SiennaSwap LP".to_string(),
        },
        Some(HOOMP_KEY),
        Some("stkd_scrt_scrt_lp_oracle"),
        "sienna-stkd-scrt-scrt-lp-oracle-v0.1",
    )?;

    Ok(stkd_scrt_scrt_lp_oracle)
}

fn deploy_sscrt_shd_lp(user_a: String, router: Contract) -> Result<SiennaswapSpotLpOracleContract> {
    let sienna_sscrt_shd_lp = Contract::new(
        SIENNA_SHD_SSCRT_POOL.to_string(),
        SIENNA_SHD_SSCRT_POOL_HASH.to_string(),
    );

    println!("Deploying SHD/SSCRT Siennaswap LP oracle.");
    let stkd_scrt_scrt_lp_oracle = SiennaswapSpotLpOracleContract::new(
        &SiennaSwapLpOracle::InitMsg {
            symbol_0: "SHD".to_string(),
            symbol_1: "SCRT".to_string(),
            router,
            exchange: sienna_sscrt_shd_lp,
            supported_key: "SHD/SSCRT SiennaSwap LP".to_string(),
        },
        Some(HOOMP_KEY),
        Some("shd_scrt_lp_oracle"),
        "sienna-shd-sscrt-lp-oracle-v0.1",
    )?;

    Ok(stkd_scrt_scrt_lp_oracle)
}

// TODO fix
fn deploy_stkd_scrt_shd_lp(
    user_a: String,
    router: Contract,
) -> Result<SiennaswapSpotLpOracleContract> {
    let sienna_stkd_scrt_shd_lp = Contract::new(
        SIENNA_STKD_SCRT_SHD_POOL.to_string(),
        SIENNA_STKD_SCRT_SHD_POOL_HASH.to_string(),
    );

    println!("Deploying STKD_SCRT/SHD Siennaswap LP oracle.");
    let stkd_scrt_scrt_lp_oracle = SiennaswapSpotLpOracleContract::new(
        &SiennaSwapLpOracle::InitMsg {
            symbol_0: "SHD".to_string(),
            symbol_1: "stkd-SCRT".to_string(),
            router,
            exchange: sienna_stkd_scrt_shd_lp,
            supported_key: "repLACE THIS".to_string(),
        },
        Some(HOOMP_KEY),
        Some("stkd_scrt_shd_lp_oracle"),
        "sienna-stkd-scrt-shd-lp-oracle-v0.1",
    )?;

    Ok(stkd_scrt_scrt_lp_oracle)
}
