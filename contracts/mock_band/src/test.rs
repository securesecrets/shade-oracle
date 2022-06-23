use crate::contract::{handle, init, query, QueryMsg};
use shade_oracles::band::{HandleMsg, InitMsg, ReferenceData};
use cosmwasm_std::{
    coins, from_binary,
    testing::{mock_dependencies, mock_env},
};
use cosmwasm_math_compat::Uint128;

#[test]
fn update_config() {
    let mut deps = mock_dependencies(20, &[]);
    let mock_coins = coins(1000, "earth");
    let env = mock_env("creator", &mock_coins);
    let msg = InitMsg {};
    let _res = init(&mut deps, env, msg).unwrap();

    let time = 1000000u64;
    // update owner
    let msg = HandleMsg::UpdateSymbolPrice {
        rate: Uint128::from(3_179_000_000_000_000_000u128),
        base_symbol: "ETH".to_string(),
        quote_symbol: "USD".to_string(),
        last_updated: Some(time),
    };

    let env = mock_env("owner0000", &mock_coins);
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(
        &deps,
        QueryMsg::GetReferenceData {
            base_symbol: "ETH".to_string(),
            quote_symbol: "USD".to_string(),
        },
    )
    .unwrap();
    let value: ReferenceData = from_binary(&res).unwrap();
    assert_eq!(Uint128::from(3_179_000_000_000_000_000u128), value.rate);
    assert_eq!(time, value.last_updated_base);
    assert_eq!(time, value.last_updated_quote);
}
