use cosmwasm_std::Uint128;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use shade_ensemble::prelude::ContractLink;
use cosmwasm_schema::cw_serde;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use secret_toolkit::utils::Query;
use shade_protocol::Contract;

#[cw_serde]
pub struct TokenAmount<A> {
    pub token: TokenType<A>,
    pub amount: Uint128,
}

#[cw_serde]
pub enum TokenType<A> {
    CustomToken {
        contract_addr: A,
        token_code_hash: String,
    },
    NativeToken {
        denom: String,
    },
}

#[cw_serde]
pub struct TokenPairAmount<A: Clone> {
    pub pair: TokenPair<A>,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
}

#[cw_serde]
pub enum ShadeSwapQueryMsg {
    GetPairInfo,
    GetEstimatedPrice { offer: TokenAmount<Addr> },
}

impl Query for ShadeSwapQueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct PairInfoResponse {
    pub liquidity_token: Contract,
    pub factory: Contract,
    pub pair: TokenPair<Addr>,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
}

#[cw_serde]
pub struct EstimatedPriceResponse {
    pub estimated_price: Uint128,
}

#[derive(Clone, Debug, JsonSchema)]
pub struct TokenPair<A>(pub TokenType<A>, pub TokenType<A>);

/*
impl Canonize for TokenPair<Addr> {
    fn canonize(&self, api: &impl Api) -> StdResult<TokenPair<CanonicalAddr>> {
        Ok(TokenPair(self.0.canonize(api)?, self.1.canonize(api)?))
    }
}

impl Humanize for TokenPair<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<TokenPair<Addr>> {
        Ok(TokenPair(self.0.humanize(api)?, self.1.humanize(api)?))
    }
}
*/

pub struct TokenPairIterator<'a, A> {
    pair: &'a TokenPair<A>,
    index: u8,
}

impl<A: Clone + PartialEq> TokenPair<A> {
    /// Returns `true` if one of the token types in the pair is the same as the argument.
    pub fn contains(&self, token: &TokenType<A>) -> bool {
        self.0 == *token || self.1 == *token
    }

    /// Returns the index of the stored token type (0 or 1) that matches the argument.
    /// Returns `None` if there are no matches.
    pub fn get_token_index(&self, token: &TokenType<A>) -> Option<usize> {
        if self.0 == *token {
            return Some(0);
        } else if self.1 == *token {
            return Some(1);
        }

        None
    }

    pub fn get_token(&self, index: usize) -> Option<&TokenType<A>> {
        match index {
            0 => Some(&self.0),
            1 => Some(&self.1),
            _ => None,
        }
    }
}

/*
impl TokenPair<Addr> {
    /// Returns the balance for each token in the pair. The order of the balances in returned array
    /// correspond to the token order in the pair i.e `[ self.0 balance, self.1 balance ]`.
    pub fn query_balances(
        &self,
        querier: &QuerierWrapper,
        exchange_addr: Addr,
        viewing_key: String,
    ) -> StdResult<[Uint128; 2]> {
        let amount_0 = self
            .0
            .query_balance(querier, exchange_addr.clone(), viewing_key.clone())?;
        let amount_1 = self.1.query_balance(querier, exchange_addr, viewing_key)?;

        // order is important
        Ok([amount_0, amount_1])
    }
}
*/

impl<A: PartialEq> PartialEq for TokenPair<A> {
    fn eq(&self, other: &TokenPair<A>) -> bool {
        (self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0)
    }
}

impl<'a, A: Clone> IntoIterator for &'a TokenPair<A> {
    type Item = &'a TokenType<A>;
    type IntoIter = TokenPairIterator<'a, A>;
    fn into_iter(self) -> Self::IntoIter {
        TokenPairIterator {
            pair: self,
            index: 0,
        }
    }
}

impl<'a, A: Clone> Iterator for TokenPairIterator<'a, A> {
    type Item = &'a TokenType<A>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(&self.pair.0),
            1 => Some(&self.pair.1),
            _ => None,
        };

        self.index += 1;

        result
    }
}

// These are only used for serde, because it doesn't work with struct tuples.
#[derive(Serialize, Deserialize)]
struct TokenPairSerde<A: Clone> {
    token_0: TokenType<A>,
    token_1: TokenType<A>,
}

impl<A: Clone + Serialize> Serialize for TokenPair<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TokenPairSerde {
            token_0: self.0.clone(),
            token_1: self.1.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de, A: Deserialize<'de> + Clone> Deserialize<'de> for TokenPair<A> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
            .map(|TokenPairSerde { token_0, token_1 }| TokenPair(token_0, token_1))
    }
}
