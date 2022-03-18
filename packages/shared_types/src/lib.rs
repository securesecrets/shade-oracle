use asset::Contract;
use scrt::{space_pad, to_binary, Coin, CosmosMsg, HumanAddr, StdResult, Uint128, WasmMsg};
use scrt_math::Uint256;

// Modules re-export
pub use fadroma::ensemble as ensemble;
pub use fadroma::auth as auth;
pub use fadroma::killswitch as killswitch;
pub use fadroma::math as scrt_math;
pub use fadroma::platform as scrt;
pub use fadroma::snip20_impl as composable_snip20;
pub use secret_toolkit;
use serde::Serialize;

pub const SECONDS_IN_A_YEAR: u128 = 31_556_952u128;

pub fn get_precision(factor: u8) -> Uint256 {
    Uint256::from(1 * 10u128.pow(factor.into()))
}

/// Normalizes amount from base decimals to normalized decimals.
pub fn normalize_token_amount(amount: u128, base_decimals: u8, normalized_decimals: u8) -> Uint256 {
    let amount = Uint256::from(amount);
    amount
        .checked_mul(get_precision(normalized_decimals))
        .unwrap()
        .checked_div(get_precision(base_decimals))
        .unwrap()
}
pub trait HandlePaddable: Serialize {
    fn to_cosmos_msg(
        &self,
        mut block_size: usize,
        contract: &Contract,
        send_amount: Option<Uint128>,
    ) -> StdResult<CosmosMsg> {
        // can not have block size of 0
        if block_size == 0 {
            block_size = 1;
        }
        let mut msg = to_binary(self)?;
        space_pad(&mut msg.0, block_size);
        let mut send = Vec::new();
        if let Some(amount) = send_amount {
            send.push(Coin {
                amount,
                denom: String::from("uscrt"),
            });
        }
        let execute = WasmMsg::Execute {
            msg,
            contract_addr: HumanAddr(contract.address.clone()),
            callback_code_hash: contract.code_hash.clone(),
            send,
        };
        Ok(execute.into())
    }
}

pub mod asset;
pub mod protocols;
pub mod querier;
pub mod rebase;
pub mod storage;
pub mod tokens;
pub mod handler;
