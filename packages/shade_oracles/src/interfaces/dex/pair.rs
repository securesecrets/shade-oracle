//! Pair oracles calculate the price of 1 liquidity pair token.

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
    use crate::interfaces::common::config::CommonConfigResponse;

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
        SetPairs(Vec<RawPairData>),
        RemovePairs(Vec<String>),
        UpdateAssets(Vec<RawAsset>),
        UpdateConfig(RawContract),
        SetStatus(bool),
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {
        #[returns(PriceResponse)]
        GetPrice { key: String },
        #[returns(PricesResponse)]
        GetPrices { keys: Vec<String> },
        #[returns(CommonConfigResponse)]
        GetConfig {},
        #[returns(PairsResponse)]
        GetPairs {},
    }

    pub type PairsResponse = Vec<PairData>;

    #[cw_serde]
    pub struct RawPairData {
        pub key: String,
        pub token_0: RawAsset,
        pub token_1: RawAsset,
        pub pair: RawContract,
    }

    #[cw_serde]
    pub struct PairData {
        pub key: String,
        pub token_0: Asset,
        pub token_1: Asset,
        pub pair: Contract,
    }
}

#[cfg(feature = "dex")]
pub use msg::*;
#[cfg(feature = "dex")]
pub use state::*;
#[cfg(feature = "dex")]
mod state {
    use std::cmp::min;

    use crate::{
        asset::Assets,
        interfaces::{
            band::ReferenceData,
            common::{config::CommonConfig, OraclePrice},
        },
        math::{FairLpPriceInfo, LiquidityPoolMath},
    };

    use super::*;
    use cosmwasm_std::{Api, QuerierWrapper, StdResult, Storage, Uint128};
    use shade_protocol::snip20::helpers::TokenInfo;

    #[cw_serde]
    /// Pair is the LP pair & symbol is the key we'll use to
    /// get the price of 1 side of the LP pair from our oracle router.
    pub struct StoredPairData {
        pub token_0: Addr,
        pub token_1: Addr,
        pub pair: Contract,
    }

    #[cw_serde]
    pub struct LiquidityPairOracle {
        pub config: CommonConfig,
    }

    impl ItemStorage for LiquidityPairOracle {
        const ITEM: Item<'static, Self> = Item::new("liquidity_pair_market_oracle");
    }

    impl<'a> LiquidityPairOracle {
        pub const ASSETS: Assets<'static, 'a> = Assets::new("pair_assets");
        // Keyed by its symbol.
        pub const PAIRS: Map<'static, &'a str, StoredPairData> = Map::new("markets");
    }

    impl LiquidityPairOracle {
        pub fn remove_keys(storage: &mut dyn Storage, keys: Vec<String>) -> StdResult<()> {
            let mut supported_keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
            for key in keys {
                if let Some(pos) = supported_keys.iter().position(|k| key.eq(k)) {
                    Self::PAIRS.remove(storage, &key);
                    supported_keys.swap_remove(pos);
                }
            }
            CommonConfig::SUPPORTED_KEYS.save(storage, &supported_keys)?;
            Ok(())
        }

        pub fn update_asset_symbol(
            &self,
            storage: &mut dyn Storage,
            api: &dyn Api,
            querier: &QuerierWrapper,
            asset: RawAsset,
        ) -> StdResult<()> {
            let asset = asset.into_asset(&self.config.router, &querier, api)?;
            Self::ASSETS.update_existing_asset(
                storage,
                &querier,
                &self.config.router,
                &asset.contract.address,
                &asset.quote_symbol,
            )?;
            Ok(())
        }

        pub fn set_pair_data(
            storage: &mut dyn Storage,
            key: String,
            token_0: Asset,
            token_1: Asset,
            pair: Contract,
        ) -> StdResult<StoredPairData> {
            Self::ASSETS.may_set(storage, &token_0)?;
            Self::ASSETS.may_set(storage, &token_1)?;
            let data = StoredPairData {
                token_0: token_0.contract.address,
                token_1: token_1.contract.address,
                pair,
            };
            Self::PAIRS.save(storage, &key, &data)?;
            CommonConfig::add_supported_key(storage, &key)?;
            Ok(data)
        }

        /// Performs validation and saves the data to storage.
        pub fn validate_and_set_pair_data(
            &self,
            storage: &mut dyn Storage,
            api: &dyn Api,
            querier: &QuerierWrapper,
            data: RawPairData,
        ) -> StdResult<StoredPairData> {
            let pair = data.pair.into_valid(api)?;
            let token_0 = data.token_0.into_asset(&self.config.router, querier, api)?;
            let token_1 = data.token_1.into_asset(&self.config.router, querier, api)?;
            Self::set_pair_data(storage, data.key, token_0, token_1, pair)
        }

        pub fn get_pair_data_resp(key: &String, storage: &dyn Storage) -> StdResult<PairData> {
            let data = Self::PAIRS.load(storage, key)?;
            let token_0 = Self::ASSETS.0.load(storage, &data.token_0)?;
            let token_1 = Self::ASSETS.0.load(storage, &data.token_1)?;
            Ok(PairData {
                key: key.clone(),
                token_0,
                token_1,
                pair: data.pair,
            })
        }

        pub fn get_supported_pairs(storage: &dyn Storage) -> StdResult<PairsResponse> {
            let keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
            let mut supported_pairs = vec![];
            for key in keys {
                supported_pairs.push(LiquidityPairOracle::get_pair_data_resp(&key, storage)?);
            }
            Ok(supported_pairs)
        }

        pub fn calculate_lp_token_spot_rate(
            data: msg::PairData,
            lp_token_info: TokenInfo,
            reserves_0: Uint128,
            reserves_1: Uint128,
            pair_prices: &[&OraclePrice; 2],
        ) -> StdResult<ReferenceData> {
            let total_supply = lp_token_info.total_supply.unwrap();
            let lp_token_decimals = lp_token_info.decimals;
            let price_0 = pair_prices[0].data();
            let price_1 = pair_prices[1].data();

            let a = FairLpPriceInfo {
                reserve: reserves_0.u128(),
                price: price_0.rate.u128(),
                decimals: data.token_0.decimals,
            };

            let b = FairLpPriceInfo {
                reserve: reserves_1.u128(),
                price: price_1.rate.u128(),
                decimals: data.token_1.decimals,
            };

            let rate = LiquidityPoolMath::get_lp_token_spot_price(
                a,
                b,
                total_supply.u128(),
                lp_token_decimals,
            )?;

            Ok(ReferenceData {
                rate,
                last_updated_base: min(price_0.last_updated_base, price_1.last_updated_base),
                last_updated_quote: min(price_0.last_updated_quote, price_1.last_updated_quote),
            })
        }

        /// Infers the price of an LP token based on its expected reserves.
        /// Does not work on the stableswap.
        pub fn calculate_lp_token_inferred_rate(
            data: msg::PairData,
            lp_token_info: TokenInfo,
            reserves_0: Uint128,
            reserves_1: Uint128,
            pair_prices: &[&OraclePrice; 2],
        ) -> StdResult<ReferenceData> {
            let total_supply = lp_token_info.total_supply.unwrap();
            let lp_token_decimals = lp_token_info.decimals;
            let price_0 = pair_prices[0].data();
            let price_1 = pair_prices[1].data();

            let a = FairLpPriceInfo {
                reserve: reserves_0.u128(),
                price: price_0.rate.u128(),
                decimals: data.token_0.decimals,
            };

            let b = FairLpPriceInfo {
                reserve: reserves_1.u128(),
                price: price_1.rate.u128(),
                decimals: data.token_1.decimals,
            };

            let rate = LiquidityPoolMath::get_lp_token_spot_price(
                a,
                b,
                total_supply.u128(),
                lp_token_decimals,
            )?;

            Ok(ReferenceData {
                rate,
                last_updated_base: min(price_0.last_updated_base, price_1.last_updated_base),
                last_updated_quote: min(price_0.last_updated_quote, price_1.last_updated_quote),
            })
        }
    }
}
