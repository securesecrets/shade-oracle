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
    band_mock::MockBand,
    index_oracle::IndexOracle,
    router::OracleRouter,
    proxy_band::ProxyBandOracle,
};

use shade_oracles::{
    common::Contract,
    band::{self, proxy},
    router,
    index_oracle,
};

// Add other adapters here as they come
fn index_test(
    symbol: String, 
    basket: HashMap<String, Uint128>,
    prices: HashMap<String, Uint128>,
    expected: Uint128,
) {
    let mut ensemble = ContractEnsemble::new(50);

    let reg_router = ensemble.register(Box::new(OracleRouter));
    let reg_index_oracle = ensemble.register(Box::new(IndexOracle));
    let reg_band = ensemble.register(Box::new(MockBand));
    let reg_band_proxy = ensemble.register(Box::new(ProxyBandOracle));

    let band = ensemble.instantiate(
        reg_band.id,
        &band::InitMsg { },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("band".into()),
                code_hash: reg_band.code_hash.clone(),
            }
        )
    ).unwrap();

    let band_proxy = ensemble.instantiate(
        reg_band_proxy.id,
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
                code_hash: reg_band.code_hash.clone(),
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
                code_hash: reg_band.code_hash.clone(),
            }
        )
    ).unwrap();

    let index_oracle = ensemble.instantiate(
        reg_index_oracle.id,
        &index_oracle::InitMsg {
            admins: None,
            router: Contract {
                address: router.address.clone(),
                code_hash: router.code_hash.clone(),
            },
            symbol,
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

    // Configure mock band prices
    for (sym, price) in prices {
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

}

macro_rules! index_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (symbol, basket, prices, expected) = $value;
                index_test(symbol, basket, prices, expected);
            }
        )*
    }
}
index_tests! {
    index_test_0: (
        "sUSD".to_string(),
        HashMap::from([
            ("USD".to_string(), Uint128(10u128.pow(18))), // 100%
        ]),
        HashMap::from([
            ("USD".to_string(), Uint128(10u128.pow(18))), // $1
        ]),
        Uint128(10u128.pow(18)), // $1
    ),
}
