use cosmwasm_math_compat::Uint128;
use shade_oracles::lp::{get_lp_token_spot_price, FairLpPriceInfo};

#[test]
fn test_get_lp_token_spot_price() {
    // Hardcoded data from USDC.e - WAVAX pool on Trader Joe
    // https://analytics.traderjoexyz.com/pairs/0xa389f9430876455c36478deea9769b7ca4e3ddb1
    // https://snowtrace.io/token/0xa389f9430876455c36478deea9769b7ca4e3ddb1#readContract
    let price_info_1 = FairLpPriceInfo {
        reserve: 2000000,
        price: 5429196975397921000,
        decimals: 6,
    };

    let price_info_2 = FairLpPriceInfo {
        reserve: 2000000,
        price: 5735646851000000000,
        decimals: 6,
    };

    let total_supply = 2000000;

    let spot_lp_token_price = get_lp_token_spot_price(price_info_1, price_info_2, total_supply, 18).unwrap();
    assert_eq!(Uint128::from(11164843826397921000000000000000u128), spot_lp_token_price)
}
