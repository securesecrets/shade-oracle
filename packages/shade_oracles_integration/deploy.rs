use colored::*;
use mulberry_integration_tests::{
    constants::*,
    contract_helpers::{
        oracles::{
            BandContract, EarnV1OracleContract, LpOracleContract, OracleContract,
            OracleRouterContract, ProxyBandOracleContract,
        },
        overseer::LendOverseerContract,
        token::Snip20Contract,
        TestableContract,
    },
    utils::{generate_label, get_test_account_addresses, print_header},
};
use mulberry_utils::{
    common::types::{self, Contract},
    composable_snip20::msg as snip20,
    protocols,
    scrt::ContractInstantiationInfo,
    scrt::{HumanAddr, Uint128},
};
use secretcli::secretcli::{account_address, query_contract, test_contract_handle, test_inst_init};
use serde_json::Result;
use shade_earn_v1::strategy::{self, msg::TreasuryInfo};
use shade_lend::{
    fee_router, liquidation,
    mocks::{
        mock_farm,
        mock_lp::{self, AssetContract},
    },
    overseer::{self},
    vault,
};
use shade_oracles::{band, common as common_oracles, lp::Dex, router::RegistryOperation};

fn main() -> Result<()> {
    let user_a = account_address(USER_A_KEY).unwrap_or_default();

    let mut users = vec![];
    users.push(HumanAddr::from(user_a.clone()));
4
    println!("Account A: {}", user_a.blue().clone());
    println!("");

    println!("Setting up mock band");
    let mock_band = BandContract::new(&band::InitMsg {}, None, Some("mock_band"))?;

    {
        // set quote for SCRT in USD to be $11
        let rate = Uint128(11 * 10u128.pow(18));
        mock_band.update_band_pair(("SCRT", "USD"), rate, None, None)?;
    }

    let scrt_oracle = ProxyBandOracleContract::new(
        user_a.clone(),
        ("SCRT", "USD"),
        mock_band.as_contract(),
        None,
        Some("scrt_oracle"),
    )?;
    Ok(())
}
