use cosmwasm_std::{Decimal256, Uint256};

use crate::build_oracle_error;

build_oracle_error! {OracleRouter {
    #[error("Router cannot be queried when it is deprecated.")]
    Deprecated,
    #[error("Can't add oracle to key {key} because one already exists.")]
    ExistingOracle { key: String },
    #[error("Reported price {actual} is not within expected deviation {deviation} of the expected price {expected}. Actual deviation {actual_deviation}.")]
    ProtectedPriceDeviation { actual: Uint256, expected: Uint256, deviation: Decimal256, actual_deviation: Decimal256 },
}}
