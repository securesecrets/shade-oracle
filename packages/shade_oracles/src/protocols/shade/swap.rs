use crate::asset::Asset;

use super::*;
pub struct ShadeSwapQuerier;

impl ShadeSwapQuerier {
    pub fn query_pair_info(
        querier: &QuerierWrapper,
        pair: &Contract,
    ) -> StdResult<PairInfoResponse> {
        QueryMsg::GetPairInfo {}.query(querier, pair)
    }
    pub fn query_swap_simulation(
        querier: &QuerierWrapper,
        pair: &Contract,
        token: &Contract,
        amount: Uint128,
    ) -> StdResult<SwapSimulationResponse> {
        let token = TokenType::CustomToken {
            contract_addr: token.address.clone(),
            token_code_hash: token.code_hash.clone(),
            oracle_key: None,
        };
        let offer = TokenAmount { token, amount };
        QueryMsg::SwapSimulation {
            offer,
            exclude_fee: Some(true),
        }
        .query(querier, pair)
    }
    pub fn query_stableswap_simulation(
        querier: &QuerierWrapper,
        pair: &Contract,
        amount: Uint128,
        token: &Asset,
    ) -> StdResult<SwapSimulationResponse> {
        // Stableswap assets are registered with an oracle key in the ShadeSwap pair.
        let token = TokenType::CustomToken {
            contract_addr: token.contract.address.clone(),
            token_code_hash: token.contract.code_hash.clone(),
            oracle_key: Some(token.quote_symbol.clone()),
        };
        let offer = TokenAmount { token, amount };
        QueryMsg::SwapSimulation {
            offer,
            exclude_fee: Some(true),
        }
        .query(querier, pair)
    }
}

#[cw_serde]
pub enum QueryMsg {
    GetPairInfo {},
    SwapSimulation {
        offer: TokenAmount,
        exclude_fee: Option<bool>,
    },
}

impl Query for QueryMsg {
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

impl PairInfoResponse {
    pub fn is_stableswap(&self) -> bool {
        self.pair.2
    }
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
    pub fn require_has_addresses(&self, token_0: &Addr, token_1: &Addr) -> StdResult<()> {
        self.require_has_address(token_0)?;
        self.require_has_address(token_1)?;
        Ok(())
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
            TokenType::NativeToken { .. } => Err(StdError::generic_err(
                "Pair Error: Token pair is not a snip20 pair.",
            )),
        }
    }
    pub fn eq_address(&self, address: &Addr) -> bool {
        match self {
            TokenType::CustomToken { contract_addr, .. } => contract_addr.eq(address),
            TokenType::NativeToken { .. } => false,
        }
    }
    pub fn require_address_eq(&self, address: &Addr) -> StdResult<()> {
        if self.eq_address(address) {
            Ok(())
        } else {
            Err(StdError::generic_err(format!(
                "Pair Error: Token is not the expected token {}.",
                address
            )))
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
