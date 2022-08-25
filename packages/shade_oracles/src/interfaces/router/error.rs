use crate::build_oracle_error;

build_oracle_error! {OracleRouter {
    #[error("Router cannot be queried when it is deprecated.")]
    Deprecated,
    #[error("Can't add oracle to key {key} because one already exists.")]
    ExistingOracle { key: String }
}}
