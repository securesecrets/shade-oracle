pub mod market;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cosmwasm_std::StdError;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use shade_protocol::utils::Query;
use shade_protocol::Contract;

#[cw_serde]
pub enum ShadeSwapQueryMsg {
    GetPairInfo {},
    SwapSimulation {
        offer: TokenAmount,
        exclude_fee: Option<bool>,
    },
}

impl Query for ShadeSwapQueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct PairInfoResponse {
    pub liquidity_token: Contract,
    pub factory: Option<Contract>,
    pub pair: TokenPair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
    pub fee_info: FeeInfo,
}

#[cw_serde]
pub struct SwapSimulationResponse {
    pub total_fee_amount: Uint128,
    pub lp_fee_amount: Uint128,
    pub shade_dao_fee_amount: Uint128,
    pub result: SwapResult,
    pub price: String,
}

#[cw_serde]
pub struct SwapResult {
    pub return_amount: Uint128,
}

#[cw_serde]
pub struct TokenAmount {
    pub token: TokenType,
    pub amount: Uint128,
}

#[cw_serde]
pub struct TokenPairAmount {
    pub pair: TokenPair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
}

#[cw_serde]
pub struct TokenPair(pub TokenType, pub TokenType, pub bool);

impl TokenPair {
    pub fn require_has_address(&self, address: &Addr) -> StdResult<()> {
        let is_one = self.0.eq_address(address);
        let is_two = self.1.eq_address(address);
        if is_one || is_two {
            Ok(())
        } else {
            Err(StdError::generic_err(format!(
                "Token {} is not in pair.",
                address
            )))
        }
    }
    pub fn into_contracts(&self) -> StdResult<[Contract; 2]> {
        let token_a = self.0.clone().into_contract()?;
        let token_b = self.1.clone().into_contract()?;
        Ok([token_a, token_b])
    }
}

#[cw_serde]
pub enum TokenType {
    CustomToken {
        contract_addr: Addr,
        token_code_hash: String,
        oracle_key: Option<String>,
    },
    NativeToken {
        denom: String,
        oracle_key: Option<String>,
    },
}

impl TokenType {
    pub fn is_native_token(&self) -> bool {
        match self {
            TokenType::NativeToken { .. } => true,
            TokenType::CustomToken { .. } => false,
        }
    }
    pub fn is_custom_token(&self) -> bool {
        match self {
            TokenType::NativeToken { .. } => false,
            TokenType::CustomToken { .. } => true,
        }
    }
    pub fn into_contract(&self) -> StdResult<Contract> {
        match self {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
                ..
            } => Ok(Contract::new(contract_addr, token_code_hash)),
            TokenType::NativeToken { .. } => {
                Err(StdError::generic_err("Token pair is not a snip20 pair."))
            }
        }
    }
    pub fn eq_address(&self, address: &Addr) -> bool {
        match self {
            TokenType::CustomToken { contract_addr, .. } => contract_addr.eq(address),
            TokenType::NativeToken { .. } => false,
        }
    }
}

#[cw_serde]
#[derive(Default)]
pub struct Fee {
    pub nom: u8,
    pub denom: u16,
}

#[cw_serde]
pub struct FeeInfo {
    pub shade_dao_address: Addr,
    pub lp_fee: Fee,
    pub shade_dao_fee: Fee,
    pub stable_lp_fee: Fee,
    pub stable_shade_dao_fee: Fee,
}
