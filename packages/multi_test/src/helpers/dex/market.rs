use super::*;
use crate::harness::dex::{ShadeSwapMarketOracle, SiennaSwapMarketOracle};
use shade_oracles::{interfaces::dex::market::*};

create_test_helper!(MarketOracleHelper);

impl MarketOracleHelper {
    pub fn init_shadeswap(sender: &User, app: &mut App, router: &Contract) -> Self {
        let contract = sender
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                },
                ShadeSwapMarketOracle::default(),
                "shadeswap_market_oracle",
            )
            .unwrap();
        Self(contract)
    }
    pub fn init_siennaswap(sender: &User, app: &mut App, router: &Contract) -> Self {
        let contract = sender
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                },
                SiennaSwapMarketOracle::default(),
                "siennaswap_market_oracle",
            )
            .unwrap();
        Self(contract)
    }
    pub fn set_keys(
        &self,
        sender: &User,
        app: &mut App,
        data: Vec<RawPairData>,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::SetKeys(data), &self.0)
    }
    
    pub fn query_pairs(
        &self,
        app: &App,
    ) -> StdResult<PairsResponse> {
        QueryMsg::GetPairs {  }.test_query(&self.0, app)
    }
    pub fn query_config(
        &self,
        app: &App,
    ) -> StdResult<ConfigResponse> {
        QueryMsg::GetConfig {  }.test_query(&self.0, app)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    mod shadeswap {
        #![allow(
            clippy::zero_prefixed_literal,
            clippy::inconsistent_digit_grouping,
            clippy::too_many_arguments
        )]

        use super::*;
        use crate::harness::MockShadePair;
        use mock_shade_pair::contract as mock_shade_pair;

        #[allow(clippy::too_many_arguments)]
        fn basic_market_test(
            symbol: String,
            base_peg: Option<String>,
            prices: HashMap<String, Uint128>,
            primary_symbol: String,
            primary_pool: Uint128,
            primary_decimals: u32,
            base_symbol: String,
            base_pool: Uint128,
            base_decimals: u32,
            expected: Uint128,
        ) {
            let user = User::new("superadmin");
            let mut app = App::default();

            let oracle_core = OracleCore::setup(&mut app, &user, prices, None, None, None).unwrap();

            // Setup tokens
            let primary_token = Snip20Helper::init(
                &user,
                &mut app,
                "Primary",
                primary_symbol.as_str(),
                primary_decimals as u8,
                &user.addr(),
                &None,
                &to_binary("").ok().unwrap(),
                "primary_token",
            );

            let base_token = Snip20Helper::init(
                &user,
                &mut app,
                "Base",
                base_symbol.as_str(),
                base_decimals as u8,
                &user.addr(),
                &None,
                &to_binary("").ok().unwrap(),
                "base_token",
            );

            let shade_pair = mock_shade_pair::InstantiateMsg {}
                .test_init(
                    MockShadePair::default(),
                    &mut app,
                    user.addr(),
                    "shade_pair",
                    &[],
                )
                .unwrap();

            mock_shade_pair::ExecuteMsg::MockPool {
                token_a: primary_token.clone().into(),
                amount_a: primary_pool,
                token_b: base_token.clone().into(),
                amount_b: base_pool,
            }
            .test_exec(&shade_pair, &mut app, user.addr(), &[])
            .unwrap();

            // Configure router w/ market oracle

            let market_oracle = MarketOracleHelper::init_shadeswap(
                &user,
                &mut app,
                &oracle_core.router.clone().into(),
            );

            let pair = RawPairData {
                target_token: RawAsset::new(primary_token.0.clone(), primary_symbol.clone()),
                base_token: RawAsset::new(base_token.0.clone(), base_symbol.clone()),
                key: symbol.clone(),
                pair: shade_pair.clone().into(),
            };

            market_oracle.set_keys(&user, &mut app, vec![pair]).unwrap();

            oracle_core
                .router
                .set_keys(
                    &user,
                    &mut app,
                    market_oracle.clone().0.into(),
                    vec![symbol.clone()],
                )
                .unwrap();

            let pairs = market_oracle.query_pairs(&app).unwrap();
            assert!(pairs.len() == 1);
            assert_eq!(pairs[0].key, symbol);
            assert_eq!(pairs[0].target_token.quote_symbol, primary_symbol);
            assert_eq!(pairs[0].base_token.quote_symbol, base_symbol);

            let price: PriceResponse = oracle_core.router.query_price(&app, symbol).unwrap();

            let data = price.data();
            assert_eq!(
                expected, data.rate,
                "Expected: {} Got: {}",
                expected, data.rate
            );
        }

        macro_rules! basic_market_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (symbol, base_peg, prices,
                         primary_symbol, primary_pool, primary_decimals,
                         base_symbol, base_pool, base_decimals, expected) = $value;

                    basic_market_test(
                        symbol.to_string(),
                        base_peg,
                        prices.into_iter().map(|(sym, p)| (sym.to_string(), Uint128::from(p))).collect(),
                        primary_symbol.to_string(),
                        Uint128::from(primary_pool),
                        primary_decimals,
                        base_symbol.to_string(),
                        Uint128::from(base_pool),
                        base_decimals,
                        Uint128::from(expected)
                    );
                }
            )*
        }
    }

        basic_market_tests! {
            usd_stables_same_decimals: (
                "USDT",
                None,
                vec![
                    ("USDT", 10u128.pow(18)), // $1
                    ("USDC", 10u128.pow(18)), // $1
                ],
                "USDT",
                10u128.pow(10),
                6,
                "USDC",
                10u128.pow(10),
                6,
                999901 * 10u128.pow(12), // ~$.99 (slippage)
            ),
            usd_stables_diff_decimals: (
                "USDT",
                None,
                vec![
                    ("USDT", 10u128.pow(18)), // $1
                    ("USDC", 10u128.pow(18)), // $1
                ],
                "USDT",
                10u128.pow(16),
                13,
                "USDC",
                10u128.pow(21),
                18,
                999000999000999001u128, // ~$.99 (slippage)
            ),
            shd_sscrt_mainnet_test: (
                "SHD",
                None,
                vec![
                    ("SSCRT", 979575 * 10u128.pow(12)), // $0.979575
                ],

                "SHD",
                1_025_322 * 10u128.pow(6),
                8,

                "SSCRT",
                8_677_013 * 10u128.pow(4),
                6,

                8_289_060_794_625 * 10u128.pow(6), // ~$8.28
            ),
        }
    }

    mod siennaswap {
        #![allow(
            clippy::zero_prefixed_literal,
            clippy::inconsistent_digit_grouping,
            clippy::too_many_arguments
        )]
        use super::*;
        use crate::harness::MockSiennaPair;
        use mock_sienna_pair::contract as mock_sienna_pair;

        #[allow(clippy::too_many_arguments)]
        fn basic_market_test(
            symbol: String,
            base_peg: Option<String>,
            prices: HashMap<String, Uint128>,
            primary_symbol: String,
            primary_pool: Uint128,
            primary_decimals: u32,
            base_symbol: String,
            base_pool: Uint128,
            base_decimals: u32,
            expected: Uint128,
        ) {
            let user = User::new("superadmin");
            let mut app = App::default();

            let oracle_core = OracleCore::setup(&mut app, &user, prices, None, None, None).unwrap();

            // Setup tokens
            let primary_token = Snip20Helper::init(
                &user,
                &mut app,
                "Primary",
                primary_symbol.as_str(),
                primary_decimals as u8,
                &user.addr(),
                &None,
                &to_binary("").ok().unwrap(),
                "primary_token",
            );

            let base_token = Snip20Helper::init(
                &user,
                &mut app,
                "Base",
                base_symbol.as_str(),
                base_decimals as u8,
                &user.addr(),
                &None,
                &to_binary("").ok().unwrap(),
                "base_token",
            );

            let siennaswap_pair = mock_sienna_pair::InstantiateMsg {}
                .test_init(
                    MockSiennaPair::default(),
                    &mut app,
                    user.addr(),
                    "shade_pair",
                    &[],
                )
                .unwrap();

            mock_sienna_pair::ExecuteMsg::MockPool {
                token_a: primary_token.clone().into(),
                amount_a: primary_pool,
                token_b: base_token.clone().into(),
                amount_b: base_pool,
            }
            .test_exec(&siennaswap_pair, &mut app, user.addr(), &[])
            .unwrap();

            mock_sienna_pair::ExecuteMsg::MockPool {
                token_a: primary_token.clone().into(),
                amount_a: primary_pool,
                token_b: base_token.clone().into(),
                amount_b: base_pool,
            }
            .test_exec(&siennaswap_pair, &mut app, user.addr(), &[])
            .unwrap();

            let pair = RawPairData {
                target_token: RawAsset::new(primary_token.0.clone(), primary_symbol.clone()),
                base_token: RawAsset::new(base_token.0.clone(), base_symbol.clone()),
                key: symbol.clone(),
                pair: siennaswap_pair.clone().into(),
            };

            let market_oracle = MarketOracleHelper::init_siennaswap(&user, &mut app, &oracle_core.router.clone().into());

            market_oracle.set_keys(&user, &mut app, vec![pair]).unwrap();

            oracle_core
                .router
                .set_keys(
                    &user,
                    &mut app,
                    market_oracle.clone().0.into(),
                    vec![symbol.clone()],
                )
                .unwrap();

            let pairs = market_oracle.query_pairs(&app).unwrap();
            assert!(pairs.len() == 1);
            assert_eq!(pairs[0].key, symbol);
            assert_eq!(pairs[0].target_token.quote_symbol, primary_symbol);
            assert_eq!(pairs[0].base_token.quote_symbol, base_symbol);

            let price: PriceResponse = oracle_core.router.query_price(&app, symbol).unwrap();
            let data = price.data;

            assert_eq!(
                expected, data.rate,
                "Expected: {} Got: {}",
                expected, data.rate
            );
        }

        macro_rules! basic_market_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (symbol, base_peg, prices,
                         primary_symbol, primary_pool, primary_decimals,
                         base_symbol, base_pool, base_decimals, expected) = $value;

                    basic_market_test(
                        symbol.to_string(),
                        base_peg,
                        prices.into_iter().map(|(sym, p)| (sym.to_string(), Uint128::from(p))).collect(),
                        primary_symbol.to_string(),
                        Uint128::from(primary_pool),
                        primary_decimals,
                        base_symbol.to_string(),
                        Uint128::from(base_pool),
                        base_decimals,
                        Uint128::from(expected),
                    );
                }
            )*
        }
    }

        basic_market_tests! {
            usd_stables_same_decimals: (
                "USDT",
                None,
                vec![
                    ("USDT", 10u128.pow(18)), // $1
                    ("USDC", 10u128.pow(18)), // $1
                ],
                "USDT",
                10u128.pow(10),
                6,
                "USDC",
                10u128.pow(10),
                6,
                0_999901 * 10u128.pow(12), // ~$.99 (slippage)
            ),
            usd_stables_diff_decimals: (
                "USDT",
                None,
                vec![
                    ("USDT", 10u128.pow(18)), // $1
                    ("USDC", 10u128.pow(18)), // $1
                ],
                "USDT",
                10u128.pow(16),
                13,
                "USDC",
                10u128.pow(21),
                18,
                999000999000999001u128, // ~$.99 (slippage)
            ),
            shd_sscrt_mainnet_test: (
                "SHD",
                None,
                vec![
                    ("SSCRT", 979575 * 10u128.pow(12)), // $0.979575
                ],

                "SHD",
                10253_22 * 10u128.pow(6),
                8,

                "SSCRT",
                86770_13 * 10u128.pow(4),
                6,

                8_289_060_794_625 * 10u128.pow(6), // ~$8.28
            ),
        }
    }
}
