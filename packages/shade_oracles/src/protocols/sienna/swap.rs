use super::*;
pub struct SiennaSwapQuerier;

impl SiennaSwapQuerier {
    pub fn query_pair_info(
        querier: &QuerierWrapper,
        pair: &Contract,
    ) -> StdResult<PairInfoResponse> {
        QueryMsg::PairInfo.query(querier, pair)
    }
    pub fn query_swap_simulation(
        querier: &QuerierWrapper,
        pair: &Contract,
        token: &Contract,
        amount: Uint128,
    ) -> StdResult<SimulationResponse> {
        let token = TokenType::CustomToken {
            contract_addr: token.address.clone(),
            token_code_hash: token.code_hash.clone(),
        };
        let offer = TokenTypeAmount { amount, token };
        QueryMsg::SwapSimulation { offer }.query(querier, pair)
    }
}

#[cw_serde]
pub struct PairInfoResponse {
    pub pair_info: PairInfo,
}

#[cw_serde]
pub struct PairInfo {
    pub liquidity_token: Contract,
    pub factory: Contract,
    pub pair: Pair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
}

/// Interface for querying a SiennaSwap exchange contract.
#[cw_serde]
pub enum QueryMsg {
    PairInfo,
    SwapSimulation { offer: TokenTypeAmount },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum TokenType {
    CustomToken {
        contract_addr: Addr,
        token_code_hash: String,
    },
    NativeToken {
        denom: String,
    },
}

impl TokenType {
    pub fn into_contract(&self) -> StdResult<Contract> {
        match self {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
                ..
            } => Ok(Contract::new(contract_addr.as_str(), token_code_hash)),
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
pub struct Pair {
    pub token_0: TokenType,
    pub token_1: TokenType,
}

impl Pair {
    pub fn require_has_address(&self, address: &Addr) -> StdResult<()> {
        let is_one = self.token_0.eq_address(address);
        let is_two = self.token_1.eq_address(address);
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
        let token_a = self.token_0.clone().into_contract()?;
        let token_b = self.token_1.clone().into_contract()?;
        Ok([token_a, token_b])
    }
}

#[cw_serde]
pub struct TokenTypeAmount {
    pub amount: Uint128,
    pub token: TokenType,
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
