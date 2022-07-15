use crate::contract::{execute, instantiate, query};
use cosmwasm_std::Uint128;
use cosmwasm_std::{
    coins, from_binary,
    testing::{mock_dependencies, mock_env, mock_info},
};
use shade_oracles::interfaces::band::{QueryMsg, ExecuteMsg, InstantiateMsg, ReferenceData};

#[test]
fn update_config() {
    let mut deps = mock_dependencies();
    let mock_coins = coins(1000, "earth");
    let env = mock_env();
    let user1 = mock_info("test", &vec![]);
    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), user1.clone(), msg).unwrap();

    let time = 1000000u64;
    // update owner
    let msg = ExecuteMsg::UpdateSymbolPrice {
        rate: Uint128::from(3_179_000_000_000_000_000u128),
        base_symbol: "ETH".to_string(),
        quote_symbol: "USD".to_string(),
        last_updated: Some(time),
    };

    let res = execute(deps.as_mut(), env.clone(), user1.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(
        deps.as_ref(),
        env,
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
