use cosmwasm_std::Uint128;
use shade_oracles::interfaces::lp::math::{get_fair_lp_token_price, FairLpPriceInfo};

#[test]
fn test_get_fair_lp_token_price_1() {
    // Hardcoded data from USDC.e - WAVAX pool on Trader Joe
    // https://analytics.traderjoexyz.com/pairs/0xa389f9430876455c36478deea9769b7ca4e3ddb1
    // https://snowtrace.io/token/0xa389f9430876455c36478deea9769b7ca4e3ddb1#readContract
    let price_info_1 = FairLpPriceInfo {
        reserve: 108206619407191,
        price: 10u128.pow(18),
        decimals: 6,
    };

    let price_info_2 = FairLpPriceInfo {
        reserve: 1234341142020098415888219,
        price: 87 * 10u128.pow(18),
        decimals: 18,
    };

    let total_supply = 9856618006458893260;

    let fair_lp_token_price = get_fair_lp_token_price(price_info_1, price_info_2, total_supply, 18);
    assert_eq!(
        Uint128::from(21872892226771994653562412u128),
        fair_lp_token_price.unwrap()
    )
}
