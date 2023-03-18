//! Oracle for a pegged asset whose target value is derived from an index of assets
//!

pub mod error;
pub mod msg;

pub const SIX_HOURS: u64 = 21600u64;

#[cw_serde]
pub struct IndexOracleConfig {
    /// The only supported symbol of this oracle.
    /// Represents the index asset (i.e. "SILK").
    pub symbol: String,
    pub router: Contract,
    /// The time difference between now and when the price feeds were last updated where we consider the price feeds to have gone stale.
    pub when_stale: u64,
    pub deviation_threshold: Decimal256,
}

/// Symbol of an index asset
pub type AssetSymbol = String;
/// List of all the index asset symbols
pub struct AssetSymbols;

make_btr! {
    /// The initial weight and computed fixed weight of an asset.
    AssetWeights {
        initial: Decimal256, U256, "Initial weight of the asset (out of 100%)";
        fixed: Decimal256, U256, "Fixed weight of the asset with respect to its value and percentage of peg"
    }
}

make_btr! {
    /// The peg of the basket token
    Peg {
        target: Uint256, U256, "Target value of the peg";
        value: Uint256, U256, "Peg price of the index asset";
        frozen: bool, bool, "Whether or not this value is frozen";
        last_updated: Uint64, u64, "When this value was last updated (in seconds)"
    }
}

use better_secret_math::U256;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal256, Uint256, Uint64};

use shade_protocol::Contract;
#[cfg(feature = "index")]
pub use state::*;
#[cfg(feature = "index")]
mod state {
    use std::{cmp::min, collections::HashMap};

    use super::{error::*, msg::*, *};
    use crate::{
        impl_global_status,
        interfaces::common::OraclePrice,
        interfaces::providers::ReferenceData,
        ssp::{Bincode2, GenericItemStorage, Item, ItemStorage, Map, MapStorage},
    };
    use better_secret_math::{
        common::{abs_diff, bankers_round, exp10, muldiv, muldiv18},
        U256,
    };
    use cosmwasm_std::{StdResult, Storage, Timestamp};

    impl ItemStorage for IndexOracleConfig {
        const ITEM: Item<'static, Self> = Item::new("indexconfig");
    }

    impl ItemStorage<Bincode2> for BtrPeg {
        const ITEM: Item<'static, Self, Bincode2> = Item::new("indextarget");
    }

    impl GenericItemStorage<Vec<String>> for AssetSymbols {
        const ITEM: Item<'static, Vec<String>> = Item::new("indexasset_symbols");
    }

    impl<'a> MapStorage<'a, &'a str, Bincode2> for BtrAssetWeights {
        const MAP: Map<'static, &'a str, Self, Bincode2> = Map::new("indexassetweightss");
    }

    pub type BtrBasket = HashMap<AssetSymbol, BtrAssetWeights>;

    pub struct IndexOracle {
        pub config: IndexOracleConfig,
        pub asset_symbols: Vec<String>,
        pub basket: BtrBasket,
        pub peg: BtrPeg,
    }

    impl_global_status!(IndexOracle, IndexOracleError);

    impl IndexOracle {
        pub fn require_peg_within_deviation(&self) -> StdResult<()> {
            let diff = abs_diff(self.peg.target, self.peg.value);
            let expected: U256 = self.peg.target.into();
            let deviation = Decimal256::from_ratio(diff, expected);
            if deviation > self.config.deviation_threshold {
                return Err(IndexOracleError::PegDeviation {
                    peg: self.peg.value.into(),
                    target: self.peg.target.into(),
                    deviation: deviation.into(),
                    threshold: self.config.deviation_threshold.into(),
                }
                .into());
            }
            Ok(())
        }
        pub fn load(storage: &dyn Storage) -> StdResult<Self> {
            let config = IndexOracleConfig::load(storage)?;
            let asset_symbols = AssetSymbols::load(storage)?;
            let mut basket = HashMap::new();
            for symbol in asset_symbols.as_slice() {
                let item = BtrAssetWeights::load(storage, symbol.as_str())?;
                basket.insert(symbol.to_string(), item);
            }
            let peg = BtrPeg::load(storage)?;
            Ok(Self {
                config,
                asset_symbols,
                basket,
                peg,
            })
        }
        pub fn init(
            index_symbol: String,
            router: Contract,
            when_stale: Uint64,
            weights: Vec<InitialBasketItem>,
            target: Uint256,
            deviation_threshold: Decimal256,
            time: &Timestamp,
        ) -> StdResult<Self> {
            if weights.is_empty() {
                return Err(IndexOracleError::EmptyBasket {}.into());
            }

            let mut asset_symbols: Vec<String> = vec![];
            let mut weight_sum = Decimal256::zero();
            let mut basket: BtrBasket = HashMap::new();

            for (sym, weight) in &weights {
                let basket_item = BtrAssetWeights::new((*weight).into(), U256::ZERO);
                weight_sum += weight;
                asset_symbols.push(sym.clone());
                match basket.insert(sym.to_string(), basket_item) {
                    Some(_) => {
                        return Err(IndexOracleError::RecursiveSymbol {
                            symbol: sym.to_string(),
                        }
                        .into())
                    }
                    None => continue,
                }
            }

            let sym_slice = asset_symbols.as_slice();

            for symbol in sym_slice {
                if symbol.eq(&index_symbol) {
                    return Err(IndexOracleError::RecursiveSymbol {
                        symbol: index_symbol,
                    }
                    .into());
                }
            }

            if weight_sum != Decimal256::percent(100) {
                return Err(IndexOracleError::InvalidBasketWeights { weight: weight_sum }.into());
            }

            let peg = BtrPeg::new(target.into(), target.into(), false, time.seconds());
            Ok(Self {
                config: IndexOracleConfig {
                    symbol: index_symbol,
                    router,
                    when_stale: when_stale.into(),
                    deviation_threshold,
                },
                asset_symbols,
                peg,
                basket,
            })
        }

        /// Updates the initial weights of the assets
        /// in the basket and returns the symbols of any newly added assets.
        pub fn update_basket(
            &mut self,
            new_basket: impl IntoIterator<Item = InitialBasketItem>,
        ) -> IndexOracleResult<Vec<String>> {
            // Get the old weights
            let mut weights = self
                .basket
                .iter()
                .into_iter()
                .map(|(sym, w)| (sym.to_owned(), w.initial.into()))
                .collect::<Vec<(String, Decimal256)>>();

            let mut new_symbols = vec![];

            let self_symbol = &self.config.symbol;
            // Update weights
            for (mod_sym, mod_weight) in new_basket.into_iter() {
                // Disallow recursive symbols
                if mod_sym.eq(self_symbol) {
                    return Err(IndexOracleError::RecursiveSymbol {
                        symbol: self_symbol.clone(),
                    });
                }

                let is_mod_sym_in_basket = self.asset_symbols.contains(&mod_sym);

                // gather new symbols for fetching
                // if all of the symbols don't match mod_sym then mod_sym is new
                if !is_mod_sym_in_basket && !mod_weight.is_zero() {
                    new_symbols.push(mod_sym.clone());
                    self.asset_symbols.push(mod_sym.clone());
                }

                // Symbol is to be removed and it existed before
                if is_mod_sym_in_basket && mod_weight.is_zero() {
                    // Guaranteed to find it since it's existed before
                    weights
                        .swap_remove(weights.iter().position(|(sym, _)| mod_sym.eq(sym)).unwrap());
                    self.asset_symbols.swap_remove(
                        self.asset_symbols
                            .iter()
                            .position(|sym| mod_sym.eq(sym))
                            .unwrap(),
                    );
                    match self.basket.remove(&mod_sym) {
                        None => {
                            return Err(IndexOracleError::BasketAssetNotFound { asset: mod_sym });
                        }
                        Some(_) => {}
                    }
                }

                // add new/updated weights
                if !mod_weight.is_zero() {
                    self.basket
                        .entry(mod_sym.clone())
                        .and_modify(|asset_weight| asset_weight.initial = mod_weight.into())
                        .or_insert_with(|| BtrAssetWeights::new(mod_weight.into(), U256::ZERO));
                }
            }

            // Verify new weights sum to 100%
            let weight_sum = self.basket.iter().map(|(_, w)| w.initial).sum::<U256>();

            if weight_sum != exp10(18) {
                return Err(IndexOracleError::InvalidBasketWeights {
                    weight: weight_sum.into(),
                });
            }
            Ok(new_symbols)
        }

        pub fn compute_fixed_weights(&mut self, prices: &[OraclePrice]) -> StdResult<()> {
            for price in prices {
                let asset_symbol = price.key();
                let weight = &self.basket[asset_symbol];
                let price: U256 = price.data.rate.into();
                // Can't overflow because initial weight cannot be greater than 10^19 and target will
                // always be reasonably small (we can arbitrarily say 10^30 is worst case), but
                // it will initially be at 10^18 if it is 1.05.
                let fixed_weight = muldiv(weight.initial, self.peg.value, price)?;
                self.basket
                    .entry(asset_symbol.to_string())
                    .and_modify(|weight| {
                        weight.fixed = fixed_weight;
                    });
            }
            Ok(())
        }
        pub fn rollback(
            &mut self,
            prices: &[OraclePrice],
            time: &Timestamp,
        ) -> IndexOracleResult<()> {
            if self.peg.frozen != true {
                return Err(IndexOracleError::RollbackNotFrozen {});
            }
            let now = time.seconds();
            let (new_target, last_updated_feeds) = self._compute_target(prices, now)?;
            if now - last_updated_feeds > self.config.when_stale {
                return Err(IndexOracleError::RollbackStale {
                    oldest_price: last_updated_feeds,
                });
            }
            let mut initial_weight_sum = U256::ZERO;
            let mut initial_weights = vec![];
            for price in prices {
                let asset_symbol = price.key();
                let weight = &self.basket[asset_symbol];
                let price: U256 = price.data.rate.into();
                let new_weight = muldiv(weight.fixed, price, new_target)?;
                initial_weight_sum += new_weight;
                initial_weights.push((asset_symbol, new_weight));
            }
            let target_weight_sum = exp10(18);
            for (asset_symbol, initial_weight) in initial_weights {
                let normalized_weight =
                    muldiv(initial_weight, target_weight_sum, initial_weight_sum)?;
                self.basket
                    .entry(asset_symbol.to_string())
                    .and_modify(|weight| {
                        weight.initial = normalized_weight;
                    });
            }

            self.compute_fixed_weights(prices)?;
            self.peg.frozen = false;
            self.peg.last_updated = now;
            Ok(())
        }

        pub fn compute_peg(
            &mut self,
            prices: Option<&Vec<OraclePrice>>,
            time: &Timestamp,
        ) -> StdResult<OraclePrice> {
            let now = time.seconds();
            let symbol = &self.config.symbol;
            let mut resp = OraclePrice::new(
                symbol.clone(),
                ReferenceData {
                    rate: self.peg.value.into(),
                    last_updated_base: now,
                    last_updated_quote: now,
                },
            );

            if self.peg.frozen || prices.is_none() {
                // If peg is frozen or we aren't getting price feeds from provider, we use the last calculated value of the peg as the peg price.
                if !self.peg.frozen && now - self.peg.last_updated > self.config.when_stale {
                    self.peg.frozen = true;
                }
                return Ok(resp);
            }

            // safe to unwrap cuz of above
            let prices = prices.unwrap();
            let (new_target, last_updated_feeds) = self._compute_target(prices, now)?;
            // If the price feeds have gone stale, freeze the target peg and use its last calculated value.
            if now - last_updated_feeds > self.config.when_stale {
                self.peg.frozen = true;
                Ok(resp)
            } else {
                self.peg.last_updated = now;
                self.peg.value = new_target;
                resp.data.rate = new_target.into();
                Ok(resp)
            }
        }
        pub fn save(&self, storage: &mut dyn Storage) -> IndexOracleResult<()> {
            let asset_symbols = &self.asset_symbols;
            self.config.save(storage)?;
            AssetSymbols::save(storage, asset_symbols)?;
            self.peg.save(storage)?;
            for symbol in asset_symbols.as_slice() {
                self.basket[symbol].save(storage, symbol)?;
            }
            Ok(())
        }
        fn _compute_target(&self, prices: &[OraclePrice], now: u64) -> StdResult<(U256, u64)> {
            let mut new_target = U256::ZERO;
            let mut last_updated_base = now;
            let mut last_updated_quote = now;
            for price in prices {
                last_updated_base = min(last_updated_base, price.data.last_updated_base);
                last_updated_quote = min(last_updated_quote, price.data.last_updated_quote);

                let asset_symbol = price.key();
                let weight = &self.basket[asset_symbol];
                let price: U256 = price.data.rate.into();
                new_target += muldiv18(weight.fixed, price)?;
            }
            let last_updated_feeds = min(last_updated_base, last_updated_quote);
            // Smooth out peg calculation to 10e-9 precision
            Ok((bankers_round(new_target, 9), last_updated_feeds))
        }
    }

    #[cfg(test)]
    #[cfg(feature = "index")]
    mod test {
        use super::{msg::InitialBasketItem, *};
        use crate::{
            interfaces::common::OraclePrice, unit_test_interface::prices::generate_price_feed,
        };
        use better_secret_math::{asserter::MathAsserter, common::exp10};

        fn basic_basket() -> Vec<InitialBasketItem> {
            vec![
                ("USD".into(), Decimal256::percent(25)),
                ("EURO".into(), Decimal256::percent(25)),
                ("GDP".into(), Decimal256::percent(25)),
                ("JPY".into(), Decimal256::percent(25)),
            ]
        }

        fn feed_0() -> Vec<OraclePrice> {
            generate_price_feed(vec![
                ("USD", "1.00", 0),
                ("EURO", "1.0196", 0),
                ("GDP", "1.208", 0),
                ("JPY", "0.0074", 0),
            ])
        }

        fn feed_1() -> Vec<OraclePrice> {
            generate_price_feed(vec![
                ("USD", "1.00", 0),
                ("EURO", "1.30", 0),
                ("GDP", "1.208", 0),
                ("JPY", "0.0074", 0),
            ])
        }

        fn feed_2() -> Vec<OraclePrice> {
            generate_price_feed(vec![
                ("USD", "1.00", 0),
                ("EURO", "0.0196", 0),
                ("GDP", "1.208", 0),
                ("JPY", "0.0074", 0),
            ])
        }

        fn feed_3() -> Vec<OraclePrice> {
            generate_price_feed(vec![
                ("USD", "1.00", 0),
                ("EURO", "1.0526", 0),
                ("GDP", "1.075", 0),
                ("JPY", "0.0094", 0),
            ])
        }

        fn basic_index_init(target: U256) -> IndexOracle {
            let timestamp = Timestamp::from_seconds(0);
            IndexOracle::init(
                "SILK".into(),
                Contract::default(),
                Uint64::new(SIX_HOURS),
                basic_basket(),
                target.into(),
                Decimal256::percent(10),
                &timestamp,
            )
            .unwrap()
        }

        #[test]
        fn index_test_1() {
            let target = U256::new(105u128) * exp10(16);
            let timestamp = Timestamp::from_seconds(0);
            let mut index_oracle = basic_index_init(target);
            index_oracle.compute_fixed_weights(&feed_0()).unwrap();
            index_oracle
                .compute_peg(Some(&feed_0()), &timestamp)
                .unwrap();

            MathAsserter::within_deviation(index_oracle.peg.value, target, exp10(16));

            index_oracle
                .compute_peg(Some(&feed_1()), &timestamp)
                .unwrap();

            let target = U256::new(112u128) * exp10(16);
            MathAsserter::within_deviation(index_oracle.peg.value, target, exp10(16));
        }

        #[test]
        fn freeze_1() {
            let target = U256::new(105u128) * exp10(16);
            let timestamp = Timestamp::from_seconds(0);
            let mut index_oracle = basic_index_init(target);
            index_oracle.compute_fixed_weights(&feed_0()).unwrap();

            index_oracle
                .compute_peg(Some(&feed_0()), &timestamp)
                .unwrap();

            MathAsserter::within_deviation(index_oracle.peg.value, target, exp10(16));

            let new_timestamp = Timestamp::from_seconds(SIX_HOURS + 10u64);

            index_oracle
                .compute_peg(Some(&feed_1()), &new_timestamp)
                .unwrap();

            assert!(index_oracle.peg.frozen);
            assert_eq!(index_oracle.peg.last_updated, 0u64);
            MathAsserter::within_deviation(index_oracle.peg.value, target, exp10(16));
        }

        #[test]
        #[cfg(feature = "index")]
        fn rollback_1() {
            let target = U256::new(105u128) * exp10(16);
            let timestamp = Timestamp::from_seconds(0);
            let mut index_oracle = basic_index_init(target);
            index_oracle.compute_fixed_weights(&feed_2()).unwrap();

            index_oracle
                .compute_peg(Some(&feed_2()), &timestamp)
                .unwrap();

            MathAsserter::within_deviation(index_oracle.peg.value, target, exp10(16));

            let new_timestamp = Timestamp::from_seconds(SIX_HOURS + 10u64);

            index_oracle
                .compute_peg(Some(&feed_3()), &new_timestamp)
                .unwrap();

            assert!(index_oracle.peg.frozen);
            assert_eq!(index_oracle.peg.last_updated, 0u64);
            MathAsserter::within_deviation(index_oracle.peg.value, target, exp10(16));

            index_oracle.rollback(&feed_3(), &timestamp).unwrap();
            index_oracle
                .compute_peg(Some(&feed_3()), &timestamp)
                .unwrap();

            assert!(!index_oracle.peg.frozen);
            assert_eq!(index_oracle.peg.last_updated, 0u64);
            MathAsserter::within_deviation(index_oracle.peg.value, target, exp10(16));
        }
    }
}
