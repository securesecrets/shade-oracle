#[cfg(not(target_arch = "wasm32"))]
pub mod harness;
#[cfg(not(target_arch = "wasm32"))]
pub mod helpers;

pub(crate) use harness::*;

#[macro_use]
pub(crate) extern crate multi_test_helpers;

pub(crate) mod mocks;
use cosmwasm_std::{to_binary, Decimal256, StdResult, Uint128, Uint256, Uint64};
use multi_test_helpers::{snip20::Snip20Helper, User};
use shade_oracles::{
    asset::RawAsset,
    interfaces::common::{PriceResponse, PricesResponse},
};
use shade_toolkit::multi_test::{App, AppResponse};

use shade_toolkit::{
    {
        {Contract, RawContract}, Query,
    },
    multi::{MultiTestable, AnyResult},
 };
use std::collections::HashMap;
