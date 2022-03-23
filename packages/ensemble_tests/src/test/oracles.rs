use std::{cell::RefCell, rc::Rc};

use crate::{constants::*, contract_helpers::oracles::*, init_oracle, init_band};
use serde_json::Result;
use shade_oracles::*;
use mulberry_utils::{
    common::types::Contract,
    ensemble::{ContractEnsemble, MockEnv},
    scrt::{ContractLink, Uint128},
};

#[test]
fn test_proxy_band_oracle() {
    let ensemble = Rc::new(RefCell::new(ContractEnsemble::new(50)));
    let scrt_rate = Uint128(918 * 10u128.pow(16));
    let avax_rate = Uint128(82 * 10u128.pow(18));
    let usdc_rate = Uint128(1 * 10u128.pow(18));

    init_band!(mock_band, "Mock band", ensemble, (SCRT_USD, scrt_rate), (AVAX_USD, avax_rate), (USDC_USD, usdc_rate));
    init_oracle!(proxy_band, scrt_oracle, "SCRT_Oracle", ensemble, SCRT_USD, mock_band);
    init_oracle!(proxy_band, avax_oracle, "AVAX_Oracle", ensemble, AVAX_USD, mock_band);
    let resp = scrt_oracle.query_price().unwrap();
    let resp2 = avax_oracle.query_price().unwrap();

    assert_eq!(resp.rate, scrt_rate);
    assert_eq!(resp2.rate, avax_rate);

    let new_scrt_to_usd_rate = Uint128(2092 * 10u128.pow(16));
    mock_band
        .update_band_pair(SCRT_USD, new_scrt_to_usd_rate, None, None)
        .unwrap();
    let resp = scrt_oracle.query_price().unwrap();
    assert_eq!(resp.rate, new_scrt_to_usd_rate);
}
