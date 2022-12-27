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

        #[allow(clippy::from_over_into)]
        impl Into<cosmwasm_std::StdError> for [<$contract Error>] {
            fn into(self) -> cosmwasm_std::StdError {
                cosmwasm_std::StdError::generic_err(self.to_string())
            }
        }

        pub type [<$contract Result>]<T> = core::result::Result<T, [<$contract Error>]>;
    }
};
}
