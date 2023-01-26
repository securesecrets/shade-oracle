use super::*;
use crate::harness::derivatives::StrideStakingDerivativesOracle;
use shade_oracles::interfaces::{common::config::CommonConfigResponse, derivatives::stride::*};

create_test_helper!(StrideStakingDerivativesOracleHelper);

impl StrideStakingDerivativesOracleHelper {
    pub fn init(sender: &User, app: &mut App, router: &Contract) -> Self {
        let contract = sender
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                },
                StrideStakingDerivativesOracle::default(),
                "stride_staking_derivatives_oracle",
            )
            .unwrap();
        Self(contract)
    }

    pub fn set_derivatives(
        &self,
        sender: &User,
        app: &mut App,
        derivatives: &[RawDerivativeData],
    ) -> AnyResult<AppResponse> {
        sender.exec(
            app,
            &ExecuteMsg::SetDerivatives(derivatives.to_vec()),
            &self.0,
        )
    }

    pub fn remove_derivatives(
        &self,
        sender: &User,
        app: &mut App,
        keys: &[String],
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::RemoveDerivatives(keys.to_vec()), &self.0)
    }

    pub fn update_derivatives(
        &self,
        sender: &User,
        app: &mut App,
        update: DerivativeUpdates,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::UpdateDerivatives(update), &self.0)
    }

    pub fn update_config(
        &self,
        sender: &User,
        app: &mut App,
        router: &RawContract,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::UpdateConfig(router.clone()), &self.0)
    }

    pub fn set_status(&self, sender: &User, app: &mut App, status: bool) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::SetStatus(status), &self.0)
    }

    pub fn query_derivatives(&self, app: &App) -> StdResult<DerivativesResponse> {
        QueryMsg::GetDerivatives {}.test_query(&self.0, app)
    }
    pub fn query_config(&self, app: &App) -> StdResult<CommonConfigResponse> {
        QueryMsg::GetConfig {}.test_query(&self.0, app)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use shade_oracles::{
        interfaces::{band::ReferenceData, common::OraclePrice},
        unit_test_interface::prices::PricesFixture,
    };

    #[test]
    fn test_stride_registry() {
        let prices = PricesFixture::basic_prices_2();
        let TestScenario {
            mut app,
            admin,
            user,
            router,
            ..
        } = TestScenario::new(prices);
        let app = &mut app;
        let oracle = StrideStakingDerivativesOracleHelper::init(&user, app, &router.into());
        assert!(oracle.set_status(&user, app, false).is_err());
        assert!(oracle.set_status(&admin, app, false).is_ok());

        let derivatives = vec![RawDerivativeData {
            key: "stkd-ETH".to_string(),
            underlying_key: "ETH".to_string(),
            initial_rate: Decimal256::from_str("1.1").unwrap(),
            apy: Decimal256::from_str("0.1").unwrap(),
            update_frequency: 100,
            timeout: 50,
        }];
        assert!(oracle.set_derivatives(&admin, app, &derivatives).is_err());
    }
}

//#[cfg(test)]
// mod test {
//     use shade_oracles::unit_test_interface::prices::PricesFixture;

//     use super::*;

//         #[test]
//         fn test_stride_registry() {
//             let mut keys = vec![];
//             let prices: HashMap<String, Uint128> = PricesFixture::basic_prices_2()
//                 .into_iter()
//                 .map(|(sym, p)| {
//                     keys.push(sym.to_string());
//                     (sym.to_string(), p.into())
//                 })
//                 .collect();
//             let test_prices = PricesFixture::basic_prices_2().clone();
//             let user = User::new("superadmin");

//             let random = User::new("random");

//             let mut app = &mut App::default();

//             let deps = OracleCore::setup(&mut app, &user, prices, None, None, None).unwrap();

//             let one = "stkd-BTC";
//             let stkd_btc = Snip20Helper::init(&user, app, one, one, 10, admin, &None, &to_binary("jwnad").unwrap(), "stkd-BTC");
//             let mut deps = mock_dependencies();
//             let mut_deps = deps.as_mut();
//             let asset = Asset {
//                 contract: Contract {
//                     address: Addr::unchecked("addr"),
//                     code_hash: "code_hash".to_string(),
//                 },
//                 quote_symbol: "quote_symbol".to_string(),
//                 decimals: 10u8,
//             };
//             let key = "key".to_string();
//             let _data = StakingDerivativesOracle::set_derivative_data(
//                 mut_deps.storage,
//                 key.clone(),
//                 asset.clone(),
//                 None,
//                 None,
//             )
//             .unwrap();
//             let resp =
//                 StakingDerivativesOracle::get_derivative_data_resp(&key, mut_deps.storage).unwrap();
//             assert_eq!(resp.key, key);
//             assert_eq!(resp.staking_derivative, asset);
//             assert_eq!(resp.timeout, None);
//             assert_eq!(resp.rate, None);
//         }

//         #[test]
//         fn test_set_derivative_data() {
//             let mut deps = mock_dependencies();
//             let mut_deps = deps.as_mut();
//             let asset = Asset {
//                 contract: Contract {
//                     address: Addr::unchecked("addr"),
//                     code_hash: "code_hash".to_string(),
//                 },
//                 quote_symbol: "quote_symbol".to_string(),
//                 decimals: 10u8,
//             };
//             let key = "key".to_string();
//             let data = StakingDerivativesOracle::set_derivative_data(
//                 mut_deps.storage,
//                 key.clone(),
//                 asset.clone(),
//                 Some(OraclePrice::default()),
//                 Some(0),
//             )
//             .unwrap();
//             assert_eq!(data.derivative, asset.contract.address);
//             assert_eq!(data.rate, OraclePrice::default());
//             assert_eq!(data.timeout, 0);
//             let stored_asset = StakingDerivativesOracle::ASSETS
//                 .0
//                 .load(mut_deps.storage, &data.derivative)
//                 .unwrap();
//             assert_eq!(stored_asset, asset);
//             let stored_data = StakingDerivativesOracle::DERIVATIVES
//                 .load(mut_deps.storage, &key)
//                 .unwrap();
//             assert_eq!(stored_data, data);
//             let supported_keys = CommonConfig::SUPPORTED_KEYS.load(mut_deps.storage).unwrap();
//             assert_eq!(supported_keys, vec![key]);
//         }

//         #[test]
//         fn test_remove_keys() {
//             let mut deps = mock_dependencies();
//             let mut_deps = deps.as_mut();
//             let mut supported_keys = vec![];
//             for i in 0..10 {
//                 let key = format!("key_{}", i);
//                 supported_keys.push(key.clone());
//                 let data = StoredDerivativeData {
//                     derivative: Addr::unchecked(format!("addr_{}", i)),
//                     rate: OraclePrice::default(),
//                     timeout: 0,
//                 };
//                 StakingDerivativesOracle::DERIVATIVES
//                     .save(mut_deps.storage, &key, &data)
//                     .unwrap();
//             }
//             CommonConfig::SUPPORTED_KEYS
//                 .save(mut_deps.storage, &supported_keys)
//                 .unwrap();
//             let keys = vec!["key_1".to_string(), "key_3".to_string()];
//             StakingDerivativesOracle::remove_keys(mut_deps.storage, keys).unwrap();
//             let supported_keys = CommonConfig::SUPPORTED_KEYS.load(mut_deps.storage).unwrap();
//             // key_1 and key_3 should be removed, not preserving order
//             assert_eq!(supported_keys.len(), 8);
//             assert!(supported_keys.contains(&"key_0".to_string()));
//             assert!(supported_keys.contains(&"key_2".to_string()));
//             assert!(supported_keys.contains(&"key_4".to_string()));
//             assert!(supported_keys.contains(&"key_5".to_string()));
//             assert!(supported_keys.contains(&"key_6".to_string()));
//             assert!(supported_keys.contains(&"key_7".to_string()));
//             assert!(supported_keys.contains(&"key_8".to_string()));
//             assert!(supported_keys.contains(&"key_9".to_string()));
//             assert!(!supported_keys.contains(&"key_1".to_string()));
//             assert!(!supported_keys.contains(&"key_3".to_string()));
//         }

//         #[test]
//         fn test_update_rates() {
//             let mut deps = mock_dependencies();
//             let mut_deps = deps.as_mut();
//             let mut supported_keys = vec![];
//             for i in 0..10 {
//                 let key = format!("key_{}", i);
//                 supported_keys.push(key.clone());
//                 let data = StoredDerivativeData {
//                     derivative: Addr::unchecked(format!("addr_{}", i)),
//                     rate: OraclePrice::default(),
//                     timeout: 0,
//                 };
//                 StakingDerivativesOracle::DERIVATIVES
//                     .save(mut_deps.storage, &key, &data)
//                     .unwrap();
//             }
//             CommonConfig::SUPPORTED_KEYS
//                 .save(mut_deps.storage, &supported_keys)
//                 .unwrap();
//             let rates = vec![
//                 DerivativeExchangeRate {
//                     symbol: "key_1".to_string(),
//                     price: Uint128::from(1u128),
//                 },
//                 DerivativeExchangeRate {
//                     symbol: "key_2".to_string(),
//                     price: Uint128::from(2u128),
//                 },
//                 DerivativeExchangeRate {
//                     symbol: "key_3".to_string(),
//                     price: Uint128::from(3u128),
//                 },
//             ];
//             let now = 100;
//             StakingDerivativesOracle::update_rates(mut_deps.storage, now, rates).unwrap();
//             for i in 0..10 {
//                 let key = format!("key_{}", i);
//                 let data = StakingDerivativesOracle::DERIVATIVES
//                     .load(mut_deps.storage, &key)
//                     .unwrap();
//                 if i == 1 {
//                     assert_eq!(data.rate.data.rate, Uint128::from(1u128));
//                     assert_eq!(data.rate.data.last_updated_base, now);
//                     assert_eq!(data.rate.data.last_updated_quote, now);
//                 } else if i == 2 {
//                     assert_eq!(data.rate.data.rate, Uint128::from(2u128));
//                     assert_eq!(data.rate.data.last_updated_base, now);
//                     assert_eq!(data.rate.data.last_updated_quote, now);
//                 } else if i == 3 {
//                     assert_eq!(data.rate.data.rate, Uint128::from(3u128));
//                     assert_eq!(data.rate.data.last_updated_base, now);
//                     assert_eq!(data.rate.data.last_updated_quote, now);
//                 } else {
//                     assert_eq!(data.rate, OraclePrice::default());
//                 }
//             }
//         }

//         #[test]
//         fn test_get_supported_derivatives() {
//             let mut deps = mock_dependencies();
//             let mut_deps = deps.as_mut();
//             // Populate storage with 10 derivatives using the set_derivative_data function
//             // and check that the get_supported_derivatives function returns the same
//             for i in 0..10 {
//                 let key = format!("key_{}", i);
//                 let asset = Asset::new(
//                     Contract::new(&Addr::unchecked(key.clone()), &"symbol".to_string()),
//                     10,
//                     key.clone(),
//                 );
//                 StakingDerivativesOracle::set_derivative_data(
//                     mut_deps.storage,
//                     key.clone(),
//                     asset,
//                     Some(OraclePrice::default()),
//                     None,
//                 )
//                 .unwrap();
//             }
//             let resp =
//                 StakingDerivativesOracle::get_supported_derivatives(mut_deps.storage).unwrap();
//             assert_eq!(resp.len(), 10);
//         }
// }
