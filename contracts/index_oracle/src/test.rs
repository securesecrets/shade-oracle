/*
use crate::contract::{handle, init, query};
use mulberry_utils::{
    common::types::Contract,
    scrt::{coins, from_binary, mock_dependencies, mock_env, StdError},
};
use shade_oracles::{
    band::proxy::{ConfigResponse, HandleMsg, InitMsg},
    common::QueryMsg,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);

    let band_contract = Contract {
        address: "band0000".to_string(),
        code_hash: "band_hash0000".to_string(),
    };
    let msg = InitMsg {
        owner: "owner0000".to_string(),
        band: band_contract,
        base_symbol: "USD".to_string(),
        quote_symbol: "ETH".to_string(),
    };
    let env = mock_env("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&deps, QueryMsg::GetConfig {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner0000", value.owner.as_str());
    assert_eq!("band0000", value.band.address)
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(20, &[]);
    let mock_coins = coins(1000, "earth");
    let msg = InitMsg {
        owner: "owner0000".to_string(),
        base_symbol: "base000".to_string(),
        band: Contract {
            address: "band0000".to_string(),
            code_hash: "band_hash0000".to_string(),
        },
        quote_symbol: "quote000".to_string(),
    };
    let env = mock_env("creator", &mock_coins);

    let _res = init(&mut deps, env, msg).unwrap();

    // update owner
    let msg = HandleMsg::UpdateConfig {
        owner: Some("owner0001".to_string()),
        band: Some(Contract {
            address: "band0001".to_string(),
            code_hash: "band_hash0001".to_string(),
        }),
        base_symbol: None,
        quote_symbol: None,
    };

    let env = mock_env("owner0000", &mock_coins);
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&deps, QueryMsg::GetConfig {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner0001", value.owner.as_str());

    // Unauthorized err
    let msg = HandleMsg::UpdateConfig {
        owner: None,
        band: None,
        base_symbol: None,
        quote_symbol: None,
    };
    let env = mock_env("creator", &mock_coins);
    let res = handle(&mut deps, env, msg);
    match res {
        Err(StdError::Unauthorized { .. }) => (),
        _ => panic!("Must return unauthorized error"),
    }
}
*/
