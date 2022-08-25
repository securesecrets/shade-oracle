#[macro_export(local_inner_macros)]
macro_rules! build_oracle_error {
    ($contract:ident {$($body:tt)*}) => {
        paste::item! {
        #[derive(thiserror::Error, Debug, PartialEq)]
        pub enum [<$contract Error>] {
            #[error("{0}")]
            Std(#[from] cosmwasm_std::StdError),
            #[error("User {user} is not an authorized admin for this contract.")]
            Unauthorized { user: cosmwasm_std::Addr },
            #[error("{0}")]
            Overflow(#[from] cosmwasm_std::OverflowError),
            #[error("{0}")]
            CheckedMultiplyRatioError(#[from] cosmwasm_std::CheckedMultiplyRatioError),
            #[error("{0}")]
            CheckedFromRatioError(#[from] cosmwasm_std::CheckedFromRatioError),
            #[error("{0}")]
            DivideByZeroError(#[from] cosmwasm_std::DivideByZeroError),
            #[error("{0}")]
            ConversionOverflowError(#[from] cosmwasm_std::ConversionOverflowError),
            #[error("Operation can't be run during normal condition.")]
            Normal,
                #[error("All operations disabled except for status toggle when frozen.")]
            Frozen,
            #[error(
                "This token amount {amount} is too large. SNIP20 token amounts cannot exceed Uint128."
            )]
            TokenAmountTooLarge { amount: cosmwasm_std::Uint256 },
            #[error("This permit signed by user {addr} has been revoked.")]
            PermitRevoked { addr: cosmwasm_std::Addr },
            #[error("Unsupported symbol {symbol}.")]
            UnsupportedSymbol { symbol: String },
            $($body)*

        }
        pub type [<$contract Result>]<T> = core::result::Result<T, [<$contract Error>]>;
    }
};
}
