//! Pair oracles use a liquidity pair to determine the price of a token in that pair.
//!
use crate::asset::{Asset, RawAsset};
use crate::interfaces::common::{PriceResponse, PricesResponse};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use secret_storage_plus::{Item, ItemStorage, Map};
use shade_protocol::{utils::asset::RawContract, Contract};

pub mod msg {

    use crate::{impl_msg_callbacks, interfaces::common::config::CommonConfigResponse};

    impl_msg_callbacks!();

    use super::*;
    #[cw_serde]
    pub struct InstantiateMsg {
        pub router: RawContract,
    }

    #[cw_serde]
    pub enum ExecuteMsg {
        SetPairs(Vec<RawPairData>),
        RemovePairs(Vec<String>),
        UpdateAssets(Vec<RawAsset>),
        UpdateConfig(RawContract),
        SetStatus(bool),
    }

    #[cw_serde]
    /// If this is a market LP oracle with the pair being ETH/USDT and we want this to be an oracle for ETH:
    ///  
    /// - Base token: USDT contract with the router symbol corresponding to the USDT price.
    /// - Target token: ETH contract with router symbol corresponding to the ETH price.
    /// - Key: the oracle key supported by this pair (ex: "ETH (ShadeSwap ETH/USDT LP)").
    pub struct RawPairData {
        pub key: String,
        pub base_token: RawAsset,
        pub target_token: RawAsset,
        pub pair: RawContract,
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
    pub struct PairData {
        pub key: String,
        pub base_token: Asset,
        pub target_token: Asset,
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
        pub base_token: Addr,
        pub target_token: Addr,
        pub pair: Contract,
    }

    #[cw_serde]
    pub struct GenericLiquidityPairOracle {
        pub config: CommonConfig,
    }

    impl ItemStorage for GenericLiquidityPairOracle {
        const ITEM: Item<'static, Self> = Item::new("liquidity_pair_market_oracle");
    }

    impl<'a> GenericLiquidityPairOracle {
        pub const ASSETS: Assets<'static, 'a> = Assets::new("pair_assets");
        // Keyed by its symbol.
        pub const PAIRS: Map<'static, &'a str, StoredPairData> = Map::new("pairs");
    }

    impl GenericLiquidityPairOracle {
        pub fn remove_keys(storage: &mut dyn Storage, keys: Vec<String>) -> StdResult<()> {
            let mut supported_keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
            for key in keys {
                if supported_keys.remove(&key) {
                    Self::PAIRS.remove(storage, &key);
                }
            }
            CommonConfig::SUPPORTED_KEYS.save(storage, &supported_keys)?;
            Ok(())
        }

        pub fn set_pair_data(
            storage: &mut dyn Storage,
            key: String,
            base_token: Asset,
            target_token: Asset,
            pair: Contract,
        ) -> StdResult<StoredPairData> {
            Self::ASSETS.may_set(storage, &base_token)?;
            Self::ASSETS.may_set(storage, &target_token)?;
            let data = StoredPairData {
                base_token: base_token.contract.address,
                target_token: target_token.contract.address,
                pair,
            };
            Self::PAIRS.save(storage, &key, &data)?;
            CommonConfig::add_supported_key(storage, &key)?;
            Ok(data)
        }

        pub fn update_asset_symbol(
            &self,
            storage: &mut dyn Storage,
            api: &dyn Api,
            querier: &QuerierWrapper,
            asset: RawAsset,
        ) -> StdResult<()> {
            let asset = asset.into_asset(&self.config.router, querier, api)?;
            Self::ASSETS.update_existing_asset(
                storage,
                querier,
                &self.config.router,
                &asset.contract.address,
                &asset.quote_symbol,
            )?;
            Ok(())
        }

        pub fn validate_and_set_pair_data(
            &self,
            storage: &mut dyn Storage,
            api: &dyn Api,
            querier: &QuerierWrapper,
            data: RawPairData,
            is_market: bool,
        ) -> StdResult<StoredPairData> {
            let pair = data.pair.into_valid(api)?;
            let base_token = data
                .base_token
                .into_asset(&self.config.router, querier, api)?;
            let target_token = if is_market {
                data.target_token
                    .into_asset_without_symbol_check(api, querier)
            } else {
                data.target_token
                    .into_asset(&self.config.router, querier, api)
            }?;
            Self::set_pair_data(storage, data.key, base_token, target_token, pair)
        }

        pub fn get_pair_data_resp(key: &String, storage: &dyn Storage) -> StdResult<PairData> {
            let data = Self::PAIRS.load(storage, key)?;
            let base_token = Self::ASSETS.0.load(storage, &data.base_token)?;
            let target_token = Self::ASSETS.0.load(storage, &data.target_token)?;
            Ok(PairData {
                key: key.clone(),
                base_token,
                target_token,
                pair: data.pair,
            })
        }

        pub fn get_supported_pairs(storage: &dyn Storage) -> StdResult<Vec<PairData>> {
            let keys = CommonConfig::SUPPORTED_KEYS.load(storage)?;
            let mut supported_pairs = vec![];
            for key in keys {
                supported_pairs.push(GenericLiquidityPairOracle::get_pair_data_resp(
                    &key, storage,
                )?);
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
                decimals: data.base_token.decimals,
            };

            let b = FairLpPriceInfo {
                reserve: reserves_1.u128(),
                price: price_1.rate.u128(),
                decimals: data.target_token.decimals,
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
                decimals: data.base_token.decimals,
            };

            let b = FairLpPriceInfo {
                reserve: reserves_1.u128(),
                price: price_1.rate.u128(),
                decimals: data.target_token.decimals,
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
