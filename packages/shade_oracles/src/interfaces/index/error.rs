use cosmwasm_std::Decimal256;

use crate::build_oracle_error;

build_oracle_error! {IndexOracle {
    #[error("Oracle cannot be queried when it is deprecated.")]
    Deprecated,
    #[error("Recursive symbol {symbol} in basket.")]
    RecursiveSymbol { symbol: String },
    #[error("Initial basket weights cannot be greater than 100%. Currently {weight}.")]
    InvalidBasketWeights { weight: Decimal256 },
    #[error("The basket cannot be empty.")]
    EmptyBasket,
    #[error("Cannot remove symbol that does not exist: {asset}.")]
    BasketAssetNotFound { asset: String },
    #[error("Cannot modify basket or target when peg is frozen")]
    FrozenPeg,
    #[error("Cannot rollback oracle if it isn't frozen.")]
    RollbackNotFrozen,
    #[error("Prices are still stale so oracle cannot be rolled back. (Oldest price is {oldest_price}.)")]
    RollbackStale { oldest_price: u64 },
    #[error("Peg {peg} has deviated too far from the last_peg value {last_value}. {deviation} > {threshold}.")]
    PegDeviation { peg: Decimal256, last_value: Decimal256, deviation: Decimal256, threshold: Decimal256 },
}}
