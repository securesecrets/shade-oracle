use crate::asset::{Asset, RawAsset};
use crate::interfaces::common::{PriceResponse, PricesResponse};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use secret_storage_plus::{Item, ItemStorage, Map};
use shade_protocol::{
    utils::{asset::RawContract, InstantiateCallback},
    Contract, BLOCK_SIZE,
};

pub mod msg {
    use super::*;
    #[cw_serde]
    pub struct InstantiateMsg {
        pub router: RawContract,
    }

    impl InstantiateCallback for InstantiateMsg {
        const BLOCK_SIZE: usize = BLOCK_SIZE;
    }

    #[cw_serde]
    pub enum ExecuteMsg {
        /// If the pair is ETH/USDT and we want this to be an oracle for ETH:
        ///  
        /// - Base symbol: the router symbol corresponding to the USDT price.
        /// - Underlying symbol: the router symbol corresponding to the ETH price.
        /// - Key: the oracle key supported by this pair (ex: "ETH (ShadeSwap ETH/USDT LP)").
        SetKey {
            key: String,
            base_asset: RawAsset,
            target_asset: String,
            pair: RawContract,
        },
        RemoveKeys {
            keys: Vec<String>,
        },
        UpdateConfig {
            new_router: RawContract,
        },
        SetStatus {
            status: bool,
        },
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {
        #[returns(PriceResponse)]
        GetPrice { key: String },
        #[returns(PricesResponse)]
        GetPrices { keys: Vec<String> },
        #[returns(ConfigResponse)]
        GetConfig {},
        #[returns(MarketsResponse)]
        GetMarkets {},
    }

    #[cw_serde]
    pub struct ConfigResponse {
        pub router: Contract,
        pub enabled: bool,
    }

    #[cw_serde]
    pub struct MarketsResponse {
        pub markets: Vec<MarketDataResponse>,
    }

    #[cw_serde]
    pub struct MarketDataResponse {
        pub key: String,
        pub data: MarketData,
    }

    #[cw_serde]
    /// Pair is the LP pair & symbol is the key we'll use to
    /// get the price of 1 side of the LP pair from our oracle router.
    pub struct MarketData {
        /// The token we are basing the other one off
        pub base_asset: Asset,
        /// Address of token we are interested in getting a price for (will be used for oracle router)
        pub target_asset: Addr,
        pub target_asset_decimals: u8,
        pub pair: Contract,
    }
}

#[cfg(feature = "shadeswap")]
pub use msg::*;
#[cfg(feature = "shadeswap")]
pub use state::*;
#[cfg(feature = "shadeswap")]
mod state {
    use crate::{
        interfaces::dex::shadeswap::{PairInfoResponse, ShadeSwapQueryMsg},
        querier::query_token_info,
    };

    use super::*;
    use cosmwasm_std::{Api, QuerierWrapper, StdResult, Storage};
    use shade_protocol::utils::Query;

    #[cw_serde]
    pub struct ShadeswapMarketOracle {
        pub router: Contract,
        pub enabled: bool,
    }

    impl ItemStorage for ShadeswapMarketOracle {
        const ITEM: Item<'static, Self> = Item::new("shadeswap_market_oracle");
    }

    impl<'a> ShadeswapMarketOracle {
        pub const SUPPORTED_KEYS: Item<'static, Vec<String>> = Item::new("supported_keys");
        // Keyed by its symbol.
        pub const MARKETS: Map<'static, &'a str, MarketData> = Map::new("markets");
    }

    impl ShadeswapMarketOracle {
        pub fn update_config(
            &mut self,
            api: &dyn Api,
            status: Option<bool>,
            router: Option<RawContract>,
        ) -> StdResult<()> {
            if let Some(router) = router {
                self.router = router.into_valid(api)?;
            }
            self.enabled = status.unwrap_or(self.enabled);
            Ok(())
        }
        pub fn remove_keys(storage: &mut dyn Storage, keys: Vec<String>) -> StdResult<()> {
            let mut supported_keys = Self::SUPPORTED_KEYS.load(storage)?;
            for key in keys {
                if let Some(pos) = supported_keys.iter().position(|k| key.eq(k)) {
                    Self::MARKETS.remove(storage, &key);
                    supported_keys.swap_remove(pos);
                }
            }
            Self::SUPPORTED_KEYS.save(storage, &supported_keys)?;
            Ok(())
        }

        pub fn fetch_market_data(
            &self,
            api: &dyn Api,
            querier: &QuerierWrapper,
            base_asset: RawAsset,
            target_asset: String,
            pair: RawContract,
        ) -> StdResult<MarketData> {
            let pair = pair.into_valid(api)?;

            let pair_info_response: PairInfoResponse =
                ShadeSwapQueryMsg::GetPairInfo {}.query(querier, &pair)?;

            let tokens = pair_info_response.pair;

            let base_asset = base_asset.into_asset(&self.router, querier, api)?;
            let target_asset = api.addr_validate(&target_asset)?;
            tokens.require_has_address(&base_asset.contract.address)?;
            tokens.require_has_address(&target_asset)?;

            let tokens = tokens.into_contracts()?;

            // Check which token in the pair is the target asset. One is guaranteed to be the target asset because of the check above.
            let target_asset = if tokens[0].address.eq(&target_asset) {
                &tokens[0]
            } else {
                &tokens[1]
            };

            let target_asset_info = query_token_info(target_asset, querier)?;

            let market_data = MarketData {
                base_asset,
                target_asset: target_asset.address.clone(),
                target_asset_decimals: target_asset_info.decimals,
                pair,
            };
            Ok(market_data)
        }
        pub fn set_key(
            &self,
            storage: &mut dyn Storage,
            key: String,
            data: MarketData,
        ) -> StdResult<()> {
            Self::MARKETS.save(storage, &key, &data)?;
            Self::SUPPORTED_KEYS.update(storage, |mut current_keys| -> StdResult<_> {
                if !current_keys.contains(&key) {
                    current_keys.push(key);
                }
                Ok(current_keys)
            })?;
            Ok(())
        }
    }
}
