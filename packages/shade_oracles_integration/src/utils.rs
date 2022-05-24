use crate::constants::*;
use colored::*;
use rand::{distributions::Alphanumeric, Rng};
use secretcli::{
    cli_types::{NetContract, TxQuery},
    secretcli::{account_address, test_contract_handle, test_inst_init},
};
use serde::Serialize;
use std::fmt::Display;

pub fn generate_label(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

pub fn print_header(header: &str) {
    println!("{}", header.on_blue());
}

pub fn print_warning(warn: &str) {
    println!("{}", warn.on_yellow());
}

pub fn print_contract(contract: &NetContract) {
    println!(
        "\tLabel: {}\n\tID: {}\n\tAddress: {}\n\tHash: {}",
        contract.label, contract.id, contract.address, contract.code_hash
    );
}

pub fn print_struct<Printable: Serialize>(item: Printable) {
    println!("{}", serde_json::to_string_pretty(&item).unwrap());
}

pub fn print_vec<Type: Display>(prefix: &str, vec: Vec<Type>) {
    for e in vec.iter().take(1) {
        print!("{}{}", prefix, e);
    }
    for e in vec.iter().skip(1) {
        print!(", {}", e);
    }
    println!();
}

// returns a tuple of 4 test addresses to use in integration tests
pub fn get_test_account_addresses() -> (String, String, String, String) {
    let user_a = account_address(USER_A_KEY).unwrap_or_default();
    let user_b = account_address(USER_B_KEY).unwrap_or_default();
    let user_c = account_address(USER_C_KEY).unwrap_or_default();
    let user_d = account_address(USER_D_KEY).unwrap_or_default();
    (user_a, user_b, user_c, user_d)
}

pub fn test_contract_init_and_debug<Message: serde::Serialize>(
    msg: &Message,
    file: &str,
    sender: &str,
    store_gas: Option<&str>,
    init_gas: Option<&str>,
    backend: Option<&str>,
    name: Option<&str>,
) {
    let result = test_inst_init(
        &msg,
        file,
        &*generate_label(8),
        sender,
        store_gas,
        init_gas,
        backend,
        name,
    );

    match result {
        Ok(contract) => {
            println!("Contract address: {}", contract.address);
            println!("Contract code hash: {}", contract.code_hash);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}

pub fn test_contract_handle_and_debug<Message: serde::Serialize>(
    msg: &Message,
    contract: &NetContract,
    sender: &str,
    gas: Option<&str>,
    backend: Option<&str>,
    amount: Option<&str>,
) {
    let result = test_contract_handle(&msg, contract, sender, gas, backend, amount);

    match result {
        Ok((compute, query)) => {
            println!("{} {}", query.gas_used, query.gas_wanted);
            println!("ComputeResponse {}", compute.input);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}

pub fn assert_handle_failure(tx_query: TxQuery) -> bool {
    tx_query.raw_log.contains("failed to execute message")
}
