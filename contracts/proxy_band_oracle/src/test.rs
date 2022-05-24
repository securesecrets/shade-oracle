use crate::contract::{handle, init, query};
use shade_oracles::{
    band::proxy::{ConfigResponse, HandleMsg, InitMsg},
    common::QueryMsg, scrt::HumanAddr,
};
use shade_oracles::{
    common::Contract,
    scrt::{coins, from_binary, mock_dependencies, mock_env, StdError},
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);

    let band_contract = Contract {
        address: HumanAddr("band0000".to_string()),
        code_hash: "band_hash0000".to_string(),
    };
    let msg = InitMsg {
        owner: HumanAddr("owner0000".to_string()),
        band: band_contract,
        quote_symbol: "USD".to_string(),
    };
    let env = mock_env("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&deps, QueryMsg::GetConfig {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner0000", value.owner.as_str());
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(20, &[]);
    let mock_coins = coins(1000, "earth");
    let msg = InitMsg {
        owner: HumanAddr("owner0000".to_string()),
        band: Contract {
            address: HumanAddr("band0000".to_string()),
            code_hash: "band_hash0000".to_string(),
        },
        quote_symbol: "quote000".to_string(),
    };
    let env = mock_env("creator", &mock_coins);

    let _res = init(&mut deps, env, msg).unwrap();

    // update owner
    let msg = HandleMsg::UpdateConfig {
        owner: Some(HumanAddr("owner0001".to_string())),
        band: Some(Contract {
            address: HumanAddr("band0001".to_string()),
            code_hash: "band_hash0001".to_string(),
        }),
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
        quote_symbol: None,
    };
    let env = mock_env("creator", &mock_coins);
    let res = handle(&mut deps, env, msg);
    match res {
        Err(StdError::Unauthorized { .. }) => (),
        _ => panic!("Must return unauthorized error"),
    }
}
