use cosmwasm_schema::cw_serde;
use crate::common::Contract;


#[cw_serde]
pub struct InitMsg {
    pub owner: String,
    pub deposit_token_oracle: Contract,
    pub strategy: Contract,
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
