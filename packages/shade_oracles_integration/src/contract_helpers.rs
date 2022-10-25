use crate::constants::{BACKEND, GAS, STORE_GAS, USER_A_KEY};
use secretcli::{
    cli_types::NetContract,
    secretcli::{init_cache, test_contract_handle},
    utils::generate_label,
};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use shade_oracles::core::{Addr, Contract};

pub mod oracles;

#[derive(Serialize, Deserialize)]
pub struct GasLog {
    pub txhash: String,
    pub gas_wanted: String,
    pub gas_used: String,
    pub timestamp: String,
}

pub trait TestableContract {
    fn get_info(&self) -> &NetContract;
    fn get_file() -> &'static str;
    fn as_contract(&self) -> Contract {
        let net = self.get_info();
        Contract {
            address: Addr::unchecked(net.address.clone()),
            code_hash: net.code_hash.clone(),
        }
    }
    fn wrap_execute<Message: serde::Serialize>(
        &self,
        msg: &Message,
        sender_key: Option<&str>,
    ) -> Result<GasLog> {
        let result = test_contract_handle(
            msg,
            self.get_info(),
            sender_key.unwrap_or(USER_A_KEY),
            Some(GAS),
            Some(BACKEND),
            None,
        )?
        .1;
        Ok(GasLog {
            txhash: result.txhash,
            gas_wanted: result.gas_wanted,
            gas_used: result.gas_used,
            timestamp: result.timestamp,
        })
    }
    fn wrap_init<Message: serde::Serialize>(
        msg: &Message,
        account_key: Option<&str>,
        name: Option<&str>,
        label: Option<&str>,
    ) -> Result<NetContract> {
        let label = match label {
            Some(label) => label.to_string(),
            None => generate_label(16),
        };
        init_cache(
            msg,
            Self::get_file(),
            &label,
            account_key.unwrap_or(USER_A_KEY),
            Some(STORE_GAS),
            Some(GAS),
            Some(BACKEND),
            name,
        )
    }
}
