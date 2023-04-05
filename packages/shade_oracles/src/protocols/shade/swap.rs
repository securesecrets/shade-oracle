use cosmwasm_std::Decimal256;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::asset::Asset;

use super::*;
pub struct ShadeSwapQuerier;

impl ShadeSwapQuerier {
    pub fn query_pair_info(querier: &QuerierWrapper, pair: &Contract) -> StdResult<PairInfo> {
        let resp: ShadeSwapQueryMsgResponse = QueryMsg::GetPairInfo {}.query(querier, pair)?;
        match resp {
            ShadeSwapQueryMsgResponse::GetPairInfo {
                liquidity_token,
                factory,
                pair,
                amount_0,
                amount_1,
                total_liquidity,
                contract_version,
                fee_info,
                stable_info,
            } => Ok(PairInfo {
                liquidity_token,
                factory,
                pair,
                amount_0,
                amount_1,
                total_liquidity,
                contract_version,
                fee_info,
                stable_info,
            }),
            _ => Err(StdError::generic_err("Unexpected response")),
        }
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
        let resp: ShadeSwapQueryMsgResponse = QueryMsg::SwapSimulation {
            offer,
            exclude_fee: Some(true),
        }
        .query(querier, pair)?;
        match resp {
            ShadeSwapQueryMsgResponse::SwapSimulation {
                total_fee_amount,
                lp_fee_amount,
                shade_dao_fee_amount,
                result,
                price,
            } => Ok(SwapSimulationResponse {
                total_fee_amount,
                lp_fee_amount,
                shade_dao_fee_amount,
                result,
                price,
            }),
            _ => Err(StdError::generic_err("Unexpected response")),
        }
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
pub struct PairInfo {
    pub liquidity_token: Contract,
    pub factory: Option<Contract>,
    pub pair: TokenPair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
    pub fee_info: FeeInfo,
    pub stable_info: Option<StablePairInfoResponse>,
}

impl PairInfo {
    pub fn is_stableswap(&self) -> bool {
        self.pair.2
    }
}

#[cw_serde]
pub struct StablePairInfoResponse {
    pub stable_params: StableParams,
    pub stable_token0_data: StableTokenData,
    pub stable_token1_data: StableTokenData,
    //p is optional so that the PairInfo query can still return even when the calculation of p fails
    pub p: Option<Decimal256>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StableTokenData {
    pub oracle_key: String,
    pub decimals: u8,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct CustomIterationControls {
    pub epsilon: Uint256, // assumed to have same decimals as SignedDecimal
    pub max_iter_newton: u16,
    pub max_iter_bisect: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StableParams {
    pub a: Decimal256,
    pub gamma1: Uint256,
    pub gamma2: Uint256,
    pub oracle: Contract,
    pub min_trade_size_x_for_y: Decimal256,
    pub min_trade_size_y_for_x: Decimal256,
    pub max_price_impact_allowed: Decimal256,
    pub custom_iteration_controls: Option<CustomIterationControls>,
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

#[derive(JsonSchema, Clone, Debug, PartialEq)]
pub struct TokenPair(pub TokenType, pub TokenType, pub bool);

impl Serialize for TokenPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (self.0.clone(), self.1.clone(), self.2.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TokenPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
            .map(|(token_0, token_1, is_stable)| TokenPair(token_0, token_1, is_stable))
    }
}

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
    pub nom: u64,
    pub denom: u64,
}

#[cw_serde]
pub struct FeeInfo {
    pub shade_dao_address: Addr,
    pub lp_fee: Fee,
    pub shade_dao_fee: Fee,
    pub stable_lp_fee: Fee,
    pub stable_shade_dao_fee: Fee,
}

#[cw_serde]
pub enum ShadeSwapQueryMsgResponse {
    GetPairInfo {
        liquidity_token: Contract,
        factory: Option<Contract>,
        pair: TokenPair,
        amount_0: Uint128,
        amount_1: Uint128,
        total_liquidity: Uint128,
        contract_version: u32,
        fee_info: FeeInfo,
        stable_info: Option<StablePairInfoResponse>,
    },
    SwapSimulation {
        total_fee_amount: Uint128,
        lp_fee_amount: Uint128,
        shade_dao_fee_amount: Uint128,
        result: SwapResult,
        price: String,
    },
}
