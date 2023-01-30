use super::*;

#[cw_serde]
pub enum CommonOracleError {
    UnauthorizedPermission(Addr, String),
    InvalidRouterSymbol(String),
    NotSupportedKey(String),
    AssetNotFound(Addr),
    DisabledOracle,
}

impl ToString for CommonOracleError {
    fn to_string(&self) -> String {
        match self {
            CommonOracleError::InvalidRouterSymbol(s) => {
                format!("Failed to query price for {s} in router.")
            }
            CommonOracleError::NotSupportedKey(s) => format!("{s} not in supported keys."),
            CommonOracleError::AssetNotFound(s) => format!("Asset {s} not found."),
            CommonOracleError::DisabledOracle => {
                "This oracle has been disabled. Only operation available is changing status."
                    .to_string()
            }
            CommonOracleError::UnauthorizedPermission(u, p) => {
                format!("{u} does not have permission {p}.")
            }
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<StdError> for CommonOracleError {
    fn into(self) -> StdError {
        StdError::generic_err(self.to_string())
    }
}
