#![allow(clippy::inconsistent_digit_grouping)]
use colored::*;
use secretcli::secretcli::{account_address, query};
use serde_json::Result;
use shade_oracles::core::{Contract, Uint128};
use shade_oracles::interfaces::band::ReferenceData;
use shade_oracles::interfaces::{band, router};
use shade_oracles_integration::constants::*;
use shade_oracles_integration::contract_helpers::oracles::BandContract;
use shade_oracles_integration::contract_helpers::{
    oracles::OracleRouterContract, TestableContract,
};

fn main() -> Result<()> {
    let deployed = DeployedContracts::docker();
    let user_a = account_address(DEPLOY_KEY).unwrap_or_default();

    let initial_prices = vec![
        ("SCRT", "USD", 1_08 * 10u128.pow(16)),
        ("SHD", "USD", 13_12 * 10u128.pow(16)),
        ("USDT/ETH LP", "USD", 1022398473 * 10u128.pow(16)),
        ("USDT/BTC LP", "USD", 40022398473 * 10u128.pow(16)),
        ("SILK", "USD", 1_05 * 10u128.pow(16)),
        ("BTC", "USD", 1_929_312 * 10u128.pow(16)),
        ("USDT", "USD", 1_00 * 10u128.pow(16)),
        ("ETH", "USD", 1_828_12 * 10u128.pow(16)),
    ]
    .into_iter()
    .map(|(a, b, c)| (a.to_string(), b.to_string(), Uint128::new(c)))
    .collect();

    println!("{}", std::env::current_dir().unwrap().display());
    let band = BandContract::new(
        &band::InstantiateMsg { initial_prices },
        Some(DEPLOY_KEY),
        Some("mock_band"),
        None,
    )?;

    let resp: ReferenceData = query(
        band.get_info(),
        band::QueryMsg::GetReferenceData {
            base_symbol: "SILK".to_string(),
            quote_symbol: "USD".to_string(),
        },
        None,
    )?;

    println!("SILK price is: {}", resp.rate);

    let oracle_router = deploy_router(user_a.clone(), band.as_contract(), &deployed)?;
    //let silk_oracle = deploy_silk(user_a.clone(), oracle_router.as_contract())?;

    //oracle_router.update_oracle(DEPLOY_KEY, "SILK", silk_oracle.as_contract())?;
    let shd_price = oracle_router.query_price("SHD".to_string());
    match shd_price {
        Ok(resp) => println!("SHD Price is: {}", resp.price.data().rate),
        Err(err) => println!("{}", err),
    }
    Ok(())
}

fn deploy_router(
    _user_a: String,
    band: Contract,
    deps: &DeployedContracts,
) -> Result<OracleRouterContract> {
    let admin_auth = deps.admin_auth.clone();

    println!("Deploying oracle router.");
    let router = OracleRouterContract::new(
        &router::msg::InstantiateMsg {
            admin_auth,
            band,
            quote_symbol: "USD".to_string(),
        },
        Some(DEPLOY_KEY),
        Some("oracle_router"),
        None,
    )?;

    Ok(router)
}

// #[allow(clippy::mistyped_literal_suffixes, clippy::zero_prefixed_literal)]
// fn deploy_silk(_user_a: String, router: Contract) -> Result<IndexOracleContract> {
//     println!("Deploying SILK oracle.");

//     let silk_oracle = IndexOracleContract::new(
//         &index_oracle::InstantiateMsg {
//             router,
//             symbol: "SILK".to_string(),
//             basket: vec![
//                 ("USD", 39_33 * 10u128.pow(14)), //  39.32%
//                 ("CNY", 7_13 * 10u128.pow(14)),  //  7.13%
//                 ("EUR", 15_97 * 10u128.pow(14)), // 15.97%
//                 ("JPY", 7_64 * 10u128.pow(14)),  //  7.64%
//                 ("GBP", 3_40 * 10u128.pow(14)),  //  3.4%
//                 ("CAD", 4_58 * 10u128.pow(14)),  //  4.58%
//                 ("KRW", 1_53 * 10u128.pow(14)),  //  1.53%
//                 ("AUD", 2_32 * 10u128.pow(14)),  //  2.32%
//                 ("IDR", 2_50 * 10u128.pow(14)),  //  2.5%
//                 ("CHF", 4_44 * 10u128.pow(14)),  //  4.44%
//                 ("SEK", 0_84 * 10u128.pow(14)),  //  0.84%
//                 ("NOK", 0_82 * 10u128.pow(14)),  //  0.82%
//                 ("SGD", 2_50 * 10u128.pow(14)),  //  2.5%
//                 ("XAU", 5_00 * 10u128.pow(14)),  //  5.0%
//                 ("WBTC", 2_00 * 10u128.pow(14)), //  2.0%
//             ]
//             .into_iter()
//             .map(|(sym, w)| (sym.to_string(), Uint128::from(w)))
//             .collect(),
//             target: Uint128::from(1_05 * 10u128.pow(16)), // $1.05
//             only_band: true,
//         },
//         Some(DEPLOY_KEY),
//         Some("silk-oracle"),
//     )?;

//     Ok(silk_oracle)
// }
