use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cosmwasm_std::Uint128;
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::Query;

#[cw_serde]
pub struct SiennaSwapPairInfoResponse {
    pub pair_info: SiennaSwapPairInfo,
}

#[cw_serde]
pub struct SiennaSwapPairInfo {
    pub liquidity_token: Contract,
    pub factory: Contract,
    pub pair: Pair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
}

#[cw_serde]
pub enum SiennaSwapExchangeQueryMsg {
    PairInfo,
    SwapSimulation { offer: TokenTypeAmount },
}

impl Query for SiennaSwapExchangeQueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum SiennaDexTokenType {
    CustomToken {
        contract_addr: Addr,
        token_code_hash: String,
    },
    NativeToken {
        denom: String,
    },
}

#[cw_serde]
pub struct Pair {
    pub token_0: SiennaDexTokenType,
    pub token_1: SiennaDexTokenType,
}

#[cw_serde]
pub struct TokenTypeAmount {
    pub amount: Uint128,
    pub token: SiennaDexTokenType,
}

#[cw_serde]
pub struct SwapSimulation {
    pub offer: TokenTypeAmount,
}

#[cw_serde]
pub struct SimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub commission_amount: Uint128,
}
