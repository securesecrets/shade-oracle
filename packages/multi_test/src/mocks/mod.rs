use super::*;

pub use shade_stkd_scrt::*;
pub use shadeswap::*;
pub use sienna_pair::*;

macro_rules! create_harness {
    ($mod_name:ident, $name:ident, $contract_path:path, $contract:ident) => {
        pub use $mod_name::*;
        mod $mod_name {
            use $contract_path;
            multi_derive::implement_multi!($name, $contract);
        }
    };
}

mod shade_stkd_scrt {
    use super::*;
    use oracle_mocks::shade_stkd_scrt::contract::*;
    use shade_oracles::{
        interfaces::derivatives::generic::RawDerivativeData,
        protocols::shade_staking_derivatives::{StakingDerivativeQueryMsg, StakingInfoResponse},
    };

    create_test_helper!(MockShadeStkdScrtHelper);

    impl MockShadeStkdScrtHelper {
        pub fn init(
            user: &User,
            app: &mut App,
            name: String,
            symbol: String,
            decimals: u8,
            price: Uint128,
        ) -> Self {
            let contract = user
                .init(
                    app,
                    &InstantiateMsg {
                        name: name.clone(),
                        symbol: symbol.clone(),
                        decimals,
                        price,
                    },
                    MockShadeStkdScrt::default(),
                    &format!("{name}-{symbol}-shade-staking-derivative"),
                )
                .unwrap();
            Self(contract)
        }
        pub fn query_staking_info(&self, app: &App) -> StakingInfoResponse {
            StakingDerivativeQueryMsg::StakingInfo { time: 0u64 }
                .test_query(&self.0, app)
                .unwrap()
        }
        pub fn to_raw_derivative_data(&self, key: &str, underlying_key: &str) -> RawDerivativeData {
            RawDerivativeData {
                key: key.to_string(),
                staking_derivative: RawAsset {
                    contract: self.0.clone().into(),
                    quote_symbol: underlying_key.to_string(),
                },
            }
        }
    }

    create_harness!(
        multi,
        MockShadeStkdScrt,
        oracle_mocks::shade_stkd_scrt,
        shade_stkd_scrt
    );
}

mod shadeswap {
    use super::*;
    use multi_test_helpers::AppResult;
    use oracle_mocks::shadeswap_pair::contract::*;
    use shade_oracles::protocols::shadeswap::{PairInfo, QueryMsg, ShadeSwapQueryMsgResponse};

    create_test_helper!(MockShadeswapPairHelper);

    impl MockShadeswapPairHelper {
        pub fn init(user: &User, app: &mut App, label: &str) -> Self {
            let contract = user
                .init(app, &InstantiateMsg {}, MockShadeswapPair::default(), label)
                .unwrap();
            Self(contract)
        }
        pub fn create_pool(
            &self,
            user: &User,
            app: &mut App,
            token_a: Contract,
            amount_a: Uint128,
            token_b: Contract,
            amount_b: Uint128,
        ) -> AppResult {
            user.exec(
                app,
                &ExecuteMsg::MockPool {
                    token_a,
                    amount_a,
                    token_b,
                    amount_b,
                },
                &self.0,
            )
        }
        pub fn query_pair_info(&self, app: &App) -> PairInfo {
            let msg: ShadeSwapQueryMsgResponse =
                QueryMsg::GetPairInfo {}.test_query(&self.0, app).unwrap();
            match msg {
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
                } => PairInfo {
                    liquidity_token,
                    factory,
                    pair,
                    amount_0,
                    amount_1,
                    total_liquidity,
                    contract_version,
                    fee_info,
                    stable_info,
                },
                _ => panic!("unexpected response"),
            }
        }
    }

    create_harness!(
        multi,
        MockShadeswapPair,
        oracle_mocks::shadeswap_pair,
        shadeswap_pair
    );
}

mod sienna_pair {
    create_harness!(
        multi,
        MockSiennaPair,
        oracle_mocks::sienna_pair,
        sienna_pair
    );
}
