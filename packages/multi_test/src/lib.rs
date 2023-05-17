#[cfg(not(target_arch = "wasm32"))]
pub mod harness;
#[cfg(not(target_arch = "wasm32"))]
pub mod helpers;

pub(crate) use harness::*;

#[macro_use]
pub(crate) extern crate multi_test_helpers;

pub(crate) mod mocks;
use cosmwasm_std::{to_binary, Addr, Decimal256, StdResult, Uint128, Uint256, Uint64};
use multi_test_helpers::{snip20::Snip20Helper, User};
use shade_oracles::{
    asset::RawAsset,
    interfaces::common::{PriceResponse, PricesResponse},
};
use shade_protocol::multi_test::{App, AppResponse};

use shade_protocol::{
    utils::{
        asset::{Contract, RawContract},
        ExecuteCallback, InstantiateCallback, MultiTestable, Query,
    },
    AnyResult,
};
use std::collections::HashMap;
