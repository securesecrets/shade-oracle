use cosmwasm_schema::cw_serde;
use shade_protocol::utils::{asset::{RawContract, Contract}, InstantiateCallback};
use crate::BLOCK_SIZE;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub deposit_token_oracle: Contract,
    pub strategy: Contract,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub deposit_token_oracle: Contract,
    pub deposit_token: Contract,
    pub share_token: Contract,
    pub strategy: Contract,
    pub enabled: bool,
}
