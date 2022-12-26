use super::*;
use crate::interfaces::common::CommonConfig;
#[cfg(feature = "core")]
use crate::ssp::{Item, ItemStorage};
use crate::BLOCK_SIZE;

pub mod market {
    use shade_protocol::{snip20::helpers::TokenInfo, utils::asset::RawContract};

    use crate::interfaces::common::InstantiateCommonConfig;

    use super::*;

    #[cw_serde]
    pub struct ConfigResponse {
        pub config: CommonConfig,
        pub market: MarketData,
    }

    #[cw_serde]
    /// Pair is the LP pair & symbol is the key we'll use to
    /// get the price of 1 side of the LP pair from our oracle router.
    pub struct MarketData {
        /// Symbol of the token we are basing the other one off (will be used for oracle router)
        pub base_peg: String,
        /// Symbol of token we are interested in getting a price for (will be used for oracle router)
        pub symbol: String,
        pub pair: Contract,
    }

    #[cfg(feature = "market-lp")]
    impl ItemStorage for MarketData {
        const ITEM: Item<'static, Self> = Item::new("marketdata");
    }

    #[cfg(feature = "market-lp")]
    pub const PRIMARY_TOKEN: Item<Contract> = Item::new("primary_token");
    #[cfg(feature = "market-lp")]
    pub const PRIMARY_INFO: Item<TokenInfo> = Item::new("primary_info");
    #[cfg(feature = "market-lp")]
    pub const BASE_INFO: Item<TokenInfo> = Item::new("base_info");

    #[cw_serde]
    pub struct InstantiateMsg {
        pub config: InstantiateCommonConfig,
        pub base_peg: Option<String>,
        pub symbol: String,
        pub pair: RawContract,
    }

    impl InstantiateCallback for InstantiateMsg {
        const BLOCK_SIZE: usize = BLOCK_SIZE;
    }
}

pub mod siennaswap {
    use cosmwasm_std::{Addr, StdError};
    use shade_protocol::utils::asset::RawContract;

    use crate::{
        interfaces::common::{CommonConfig, InstantiateCommonConfig},
        protocols::siennaswap::{SiennaDexTokenType, SiennaSwapPairInfo},
    };

    use super::*;
    /// Oracle1 - contract for an oracle of asset 1
    ///
    /// Oracle2 - contract for an oracle of asset 2
    ///
    /// Factory - contract that mints the LP token for asset 1 & asset 2
    /// (SecretSwap - Pair | SiennaSwap - Exchange)
    #[cw_serde]
    pub struct InstantiateMsg {
        pub config: InstantiateCommonConfig,
        pub symbol_0: String,
        pub symbol_1: String,
        pub exchange: RawContract,
    }

    impl InstantiateCallback for InstantiateMsg {
        const BLOCK_SIZE: usize = BLOCK_SIZE;
    }

    #[cw_serde]
    pub struct PairData {
        pub lp_token: Contract,
        pub symbol_0: String,
        pub symbol_1: String,
        pub token0_decimals: u8,
        pub token1_decimals: u8,
    }

    #[cfg(feature = "siennaswap")]
    impl ItemStorage for PairData {
        const ITEM: Item<'static, Self> = Item::new("pairdata");
    }

    #[cfg(feature = "siennaswap")]
    pub const EXCHANGE: Item<Contract> = Item::new("exchange");

    #[cw_serde]
    pub struct ConfigResponse {
        pub config: CommonConfig,
        pub exchange: Contract,
        pub pair: PairData,
    }

    #[cfg(feature = "siennaswap")]
    pub fn resolve_pair(info: &SiennaSwapPairInfo) -> StdResult<(Contract, Contract)> {
        let mut tokens: (Contract, Contract) = (
            Contract::new(&Addr::unchecked("0"), &String::default()),
            Contract::new(&Addr::unchecked("0"), &String::default()),
        );

        if let SiennaDexTokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } = &info.pair.token_0
        {
            let token0 = Contract::new(&contract_addr.clone(), token_code_hash);
            tokens.0 = token0;
        } else {
            return Err(StdError::generic_err(
                "Could not resolve SiennaSwap token 1 info.",
            ));
        }
        if let SiennaDexTokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } = &info.pair.token_1
        {
            let token1 = Contract::new(&contract_addr.clone(), token_code_hash);
            tokens.1 = token1;
        } else {
            return Err(StdError::generic_err(
                "Could not resolve SiennaSwap token 2 info.",
            ));
        }
        Ok(tokens)
    }
}
