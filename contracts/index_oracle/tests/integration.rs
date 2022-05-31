use std::collections::HashMap;
use cosmwasm_std::{
    coins, from_binary, to_binary,
    Extern, HumanAddr, StdError,
    Binary, StdResult, HandleResponse, Env,
    InitResponse, Uint128,
};

use fadroma::{
    ContractLink, 
    ensemble::{
       MockEnv, MockDeps, 
       ContractHarness, ContractEnsemble,
    },
};

use shade_oracles_ensemble::harness::{
    MockBand,
    IndexOracle,
    OracleRouter,
    ProxyBandOracle,
};

use shade_oracles::{
    common::{self, Contract, OraclePrice},
    band::{self, proxy},
    router,
    index_oracle,
};

fn basic_index_test(
    symbol: String, 
    basket: Vec<(String, Uint128)>,
    prices: HashMap<String, Uint128>,
    target: Uint128,
    expected: Uint128,
    error: Uint128,
) {
    let mut ensemble = ContractEnsemble::new(50);

    let reg_router = ensemble.register(Box::new(OracleRouter));
    let reg_index_oracle = ensemble.register(Box::new(IndexOracle));
    let reg_mock_band = ensemble.register(Box::new(MockBand));
    let reg_mock_band_proxy = ensemble.register(Box::new(ProxyBandOracle));

    let band = ensemble.instantiate(
        reg_mock_band.id,
        &band::InitMsg { },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("band".into()),
                code_hash: reg_mock_band.code_hash.clone(),
            }
        )
    ).unwrap();

    let band_proxy = ensemble.instantiate(
        reg_mock_band_proxy.id,
        &proxy::InitMsg {
            owner: HumanAddr("admin".into()),
            band: Contract {
                address: band.address.clone(),
                code_hash: band.code_hash.clone(),
            },
            quote_symbol: "USD".to_string(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("band_proxy".into()),
                code_hash: reg_mock_band.code_hash.clone(),
            }
        )
    ).unwrap();


    let router = ensemble.instantiate(
        reg_router.id,
        &router::InitMsg {
            owner: HumanAddr("admin".into()),
            default_oracle: Contract {
                address: band_proxy.address.clone(),
                code_hash: band_proxy.code_hash.clone(),
            },
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("router".into()),
                code_hash: reg_router.code_hash.clone(),
            }
        )
    ).unwrap();


    let mut operations = vec![];

    for (sym, _) in prices.clone() {
        operations.push(
            router::RegistryOperation::Add {
                oracle: Contract {
                    address: band_proxy.address.clone(),
                    code_hash: band_proxy.code_hash.clone(),
                },
                key: sym,
            }
        );
    }

    // Configure BAND symbols on router
    ensemble.execute(
        &router::HandleMsg::BatchUpdateRegistry {
            operations,
        },
        MockEnv::new(
            "admin",
            router.clone(),
        ),
    ).unwrap();

    // Configure mock band prices
    for (sym, price) in prices.clone() {
        ensemble.execute(
            &band::HandleMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: "USD".to_string(),
                rate: price,
                last_updated: None,
            },
            MockEnv::new(
                "admin", 
                band.clone(),
            ),
        ).unwrap();
    }

    let index_oracle = ensemble.instantiate(
        reg_index_oracle.id,
        &index_oracle::InitMsg {
            admins: None,
            router: Contract {
                address: router.address.clone(),
                code_hash: router.code_hash.clone(),
            },
            symbol: symbol.clone(),
            target,
            basket,
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("index".into()),
                code_hash: reg_index_oracle.code_hash.clone(),
            }
        )
    ).unwrap();

    // Configure router w/ index oracle
    ensemble.execute(
        &router::HandleMsg::UpdateRegistry {
            operation: router::RegistryOperation::Add {
                oracle: Contract {
                    address: index_oracle.address.clone(),
                    code_hash: index_oracle.code_hash.clone(),
                },
                key: symbol.clone(),
            }
        },
        MockEnv::new(
            "admin",
            router.clone(),
        ),
    ).unwrap();

    // Not sure why this query wont let me unwrap
    match ensemble.query(
        index_oracle.address.clone(),
        &index_oracle::QueryMsg::GetPrice {
            symbol: symbol.clone()
        }
    ) {
        Ok(b) => {
            let resp: OraclePrice = from_binary(&b).ok().unwrap();

            let mut err = Uint128::zero();
            if resp.price.rate > expected {
                err = (resp.price.rate - expected).ok().unwrap();
            }
            else {
                err = (expected - resp.price.rate).ok().unwrap();
            }
            let acceptable = expected.multiply_ratio(error, 10u128.pow(18));

            assert!(err <= acceptable, "price: {}, expected: {}, exceeds acceptable error", resp.price.rate, expected);
        },
        Err(e) => assert!(false, "Failed to query index {}", e.to_string()),
    };
}

macro_rules! basic_index_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (symbol, basket, prices, target, expected, error) = $value;
                basic_index_test(
                    symbol.to_string(), 
                    basket.into_iter().map(|(sym, w)| (sym.to_string(), Uint128(w))).collect(),
                    prices.into_iter().map(|(sym, p)| (sym.to_string(), Uint128(p))).collect(),
                    Uint128(target),
                    Uint128(expected),
                    Uint128(error),
                );
            }
        )*
    }
}

basic_index_tests! {
    basic_index_test_0: (
        "sUSD",
        vec![
            ("USD", 10u128.pow(18)), // 100%
        ],
        vec![
            ("USD", 10u128.pow(18)), // $1
        ],
        10u128.pow(18), // $1
        10u128.pow(18), // $1
        10u128.pow(10), // .000001% error
    ),
    basic_index_test_1: (
        "INDEX",
        vec![
            ("USD", 25 * 10u128.pow(16)), // 25%
            ("BTC", 25 * 10u128.pow(16)), // 25%
            ("ETH", 25 * 10u128.pow(16)), // 25%
            ("XAU", 25 * 10u128.pow(16)), // 25%
        ],
        vec![
            ("USD", 1_00 * 10u128.pow(14)), // $1
            ("BTC", 29_398_20 * 10u128.pow(14)), // $29398.2
            ("ETH", 1_831_26 * 10u128.pow(14)), // $1831.26
            ("XAU", 1_852_65 * 10u128.pow(14)), // $1852.65
        ],
        1 * 10u128.pow(18),
        1 * 10u128.pow(18),
        10u128.pow(10), // .000001% error
    ),
    basic_index_test_2: (
        "SILK",
        vec![
            ("USD", 39_33 * 10u128.pow(14)), //  39.32%
            ("CNY", 7_13 * 10u128.pow(14)), //  7.13%
            ("EUR", 15_97 * 10u128.pow(14)), // 15.97%
            ("JPY", 7_64 * 10u128.pow(14)), //  7.64%
            ("GBP", 3_40 * 10u128.pow(14)), //  3.4%
            ("CAD", 4_58 * 10u128.pow(14)), //  4.58%
            ("KRW", 1_53 * 10u128.pow(14)), //  1.53%
            ("AUD", 2_32 * 10u128.pow(14)), //  2.32%
            ("IDR", 2_50 * 10u128.pow(14)), //  2.5%
            ("CHF", 4_44 * 10u128.pow(14)), //  4.44%
            ("SEK", 0_84 * 10u128.pow(14)), //  0.84%
            ("NOK", 0_82 * 10u128.pow(14)), //  0.82%
            ("SGD", 2_50 * 10u128.pow(14)), //  2.5%
            ("XAU", 5_00 * 10u128.pow(14)), //  5.0%
            ("BTC", 2_00 * 10u128.pow(14)), //  2.0%
        ],
        vec![
            ("USD", 1_00 * 10u128.pow(14)), // $1
            ("CNY", 0_15 * 10u128.pow(14)), // $0.15
            ("EUR", 1_07 * 10u128.pow(14)), // $1.07
            ("JPY", 0_0079 * 10u128.pow(12)), // $0.0079
            ("GBP", 1_26 * 10u128.pow(14)), // $1.26
            ("CAD", 0_78 * 10u128.pow(14)), // $0.78
            ("KRW", 0_00079 * 10u128.pow(11)), // $0.00079
            ("AUD", 0_71 * 10u128.pow(14)), // $0.71
            ("IDR", 0_000068 * 10u128.pow(10)), // $0.000068
            ("CHF", 1_04 * 10u128.pow(14)), // $1.04
            ("SEK", 0_10 * 10u128.pow(14)), // $0.10
            ("NOK", 0_10 * 10u128.pow(14)), // $0.10
            ("SGD", 0_73 * 10u128.pow(14)), // $0.73
            ("XAU", 1_852_65 * 10u128.pow(14)), // $1852.65
            ("BTC", 29_398_20 * 10u128.pow(14)), // $29398.2
        ],
        1_05 * 10u128.pow(16),
        1_05 * 10u128.pow(16),
        10u128.pow(10), // .000001% error
    ),
}


/* - Setup oracle with symbol, basket, prices, & target -- check against expected_initial
 * - Change to new_prices & apply mod_basket changes -- check against expected_final
 */
fn mod_index_test(
    symbol: String, 
    basket: Vec<(String, Uint128)>,
    prices: HashMap<String, Uint128>,
    new_prices: HashMap<String, Uint128>,
    mod_basket: Vec<(String, Uint128)>,
    expected_basket: Vec<(String, Uint128)>,
    target: Uint128,
    expected_initial: Uint128,
    expected_final: Uint128,
    error: Uint128,
) {
    let mut ensemble = ContractEnsemble::new(50);

    let reg_router = ensemble.register(Box::new(OracleRouter));
    let reg_index_oracle = ensemble.register(Box::new(IndexOracle));
    let reg_mock_band = ensemble.register(Box::new(MockBand));
    let reg_mock_band_proxy = ensemble.register(Box::new(ProxyBandOracle));

    let band = ensemble.instantiate(
        reg_mock_band.id,
        &band::InitMsg { },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("band".into()),
                code_hash: reg_mock_band.code_hash.clone(),
            }
        )
    ).unwrap();

    let band_proxy = ensemble.instantiate(
        reg_mock_band_proxy.id,
        &proxy::InitMsg {
            owner: HumanAddr("admin".into()),
            band: Contract {
                address: band.address.clone(),
                code_hash: band.code_hash.clone(),
            },
            quote_symbol: "USD".to_string(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("band_proxy".into()),
                code_hash: reg_mock_band.code_hash.clone(),
            }
        )
    ).unwrap();


    let router = ensemble.instantiate(
        reg_router.id,
        &router::InitMsg {
            owner: HumanAddr("admin".into()),
            default_oracle: Contract {
                address: band_proxy.address.clone(),
                code_hash: band_proxy.code_hash.clone(),
            },
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("router".into()),
                code_hash: reg_router.code_hash.clone(),
            }
        )
    ).unwrap();


    let mut operations = vec![];

    for (sym, _) in prices.clone() {
        operations.push(
            router::RegistryOperation::Add {
                oracle: Contract {
                    address: band_proxy.address.clone(),
                    code_hash: band_proxy.code_hash.clone(),
                },
                key: sym,
            }
        );
    }

    // Configure BAND symbols on router
    ensemble.execute(
        &router::HandleMsg::BatchUpdateRegistry {
            operations,
        },
        MockEnv::new(
            "admin",
            router.clone(),
        ),
    ).unwrap();

    // Configure mock band prices
    for (sym, price) in prices.clone() {
        ensemble.execute(
            &band::HandleMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: "USD".to_string(),
                rate: price,
                last_updated: None,
            },
            MockEnv::new(
                "admin", 
                band.clone(),
            ),
        ).unwrap();
    }

    let index_oracle = ensemble.instantiate(
        reg_index_oracle.id,
        &index_oracle::InitMsg {
            admins: None,
            router: Contract {
                address: router.address.clone(),
                code_hash: router.code_hash.clone(),
            },
            symbol: symbol.clone(),
            target,
            basket,
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("index".into()),
                code_hash: reg_index_oracle.code_hash.clone(),
            }
        )
    ).unwrap();

    // Configure router w/ index oracle
    ensemble.execute(
        &router::HandleMsg::UpdateRegistry {
            operation: router::RegistryOperation::Add {
                oracle: Contract {
                    address: index_oracle.address.clone(),
                    code_hash: index_oracle.code_hash.clone(),
                },
                key: symbol.clone(),
            }
        },
        MockEnv::new(
            "admin",
            router.clone(),
        ),
    ).unwrap();

    // Not sure why this query wont let me unwrap
    match ensemble.query(
        index_oracle.address.clone(),
        &index_oracle::QueryMsg::GetPrice {
            symbol: symbol.clone()
        }
    ) {
        Ok(b) => {
            let resp: OraclePrice = from_binary(&b).ok().unwrap();

            let mut err = Uint128::zero();
            if resp.price.rate > expected_initial {
                err = (resp.price.rate - expected_initial).ok().unwrap();
            }
            else {
                err = (expected_initial - resp.price.rate).ok().unwrap();
            }
            let acceptable = expected_initial.multiply_ratio(error, 10u128.pow(18));

            assert!(err <= acceptable, "price: {}, expected: {}, exceeds acceptable error", resp.price.rate, expected_initial);
        },
        Err(e) => assert!(false, "Failed to query index {}", e.to_string()),
    };

    /* TODO
     * - Update new prices
     * - mod basket
     * - check final price
    */

    // Update mock band prices
    for (sym, price) in new_prices.clone() {
        ensemble.execute(
            &band::HandleMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: "USD".to_string(),
                rate: price,
                last_updated: None,
            },
            MockEnv::new(
                "admin", 
                band.clone(),
            ),
        ).unwrap();
    }

    // Check price updates
    match ensemble.query(
        index_oracle.address.clone(),
        &index_oracle::QueryMsg::GetPrice {
            symbol: symbol.clone()
        }
    ) {
        Ok(b) => {
            let resp: OraclePrice = from_binary(&b).ok().unwrap();

            let mut err = Uint128::zero();
            if resp.price.rate > expected_final {
                err = (resp.price.rate - expected_final).ok().unwrap();
            }
            else {
                err = (expected_final - resp.price.rate).ok().unwrap();
            }
            let acceptable = expected_final.multiply_ratio(error, 10u128.pow(18));

            assert!(err <= acceptable, "Price change check failed price: {}, expected: {}, exceeds acceptable error", resp.price.rate, expected_final);
        },
        Err(e) => assert!(false, "Failed to query index {}", e.to_string()),
    };

    // Update basket
    ensemble.execute(
        &index_oracle::HandleMsg::ModBasket {
            basket: mod_basket,
        },
        MockEnv::new(
            "admin",
            index_oracle.clone(),
        ),
    ).unwrap();

    // check price doesn't change on mod_price
    match ensemble.query(
        index_oracle.address.clone(),
        &index_oracle::QueryMsg::GetPrice {
            symbol: symbol.clone()
        }
    ) {
        Ok(b) => {
            let resp: OraclePrice = from_binary(&b).ok().unwrap();

            let mut err = Uint128::zero();
            if resp.price.rate > expected_final {
                err = (resp.price.rate - expected_final).ok().unwrap();
            }
            else {
                err = (expected_final - resp.price.rate).ok().unwrap();
            }
            let acceptable = expected_final.multiply_ratio(error, 10u128.pow(18));

            assert!(err <= acceptable, "Post-Mod price: {}, expected: {}, exceeds acceptable error", resp.price.rate, expected_final);
        },
        Err(e) => assert!(false, "Failed to query index {}", e.to_string()),
    };
}

macro_rules! mod_index_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (symbol, basket, prices, new_prices, mod_basket, expected_basket, target, expected_initial, expected_final, error) = $value;

                mod_index_test(
                    symbol.to_string(), 
                    basket.into_iter().map(|(sym, w)| (sym.to_string(), Uint128(w))).collect(),
                    prices.into_iter().map(|(sym, p)| (sym.to_string(), Uint128(p))).collect(),
                    new_prices.into_iter().map(|(sym, p)| (sym.to_string(), Uint128(p))).collect(),
                    mod_basket.into_iter().map(|(sym, p)| (sym.to_string(), Uint128(p))).collect(),
                    expected_basket.into_iter().map(|(sym, p)| (sym.to_string(), Uint128(p))).collect(),
                    Uint128(target),
                    Uint128(expected_initial),
                    Uint128(expected_final),
                    Uint128(error),
                );
            }
        )*
    }
}

mod_index_tests! {
    mod_index_test_0: (
        "sUSD",
        // basket
        vec![
            ("USD", 10u128.pow(18)), // 100%
        ],
        // prices
        vec![
            ("USD", 10u128.pow(18)), // $1
            ("BTC", 30 * 10u128.pow(18)), // $30,000
        ],
        // new prices
        vec![
            ("USD", 98 * 10u128.pow(16)), // $0.98
        ],
        // mod basket
        vec![
            ("USD", 0), // 0% (remove)
            ("BTC", 10u128.pow(18)), // 100%
        ],
        //expected basket
        vec![
            ("BTC", 10u128.pow(18)), // 100%
        ],
        // target
        10u128.pow(18), // $1
        // expected initial
        10u128.pow(18), // $1
        // expected final
        98 * 10u128.pow(16), // $0.98
        10u128.pow(10), // .000001% error
    ),
}
