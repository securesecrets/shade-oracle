use super::*;

#[cw_serde]
pub enum CommonOracleError {
    NotSupportedKey(String),
    AssetNotFound(Addr),
}

impl ToString for CommonOracleError {
    fn to_string(&self) -> String {
        match self {
            CommonOracleError::NotSupportedKey(s) => format!("{} not in supported keys.", s),
            CommonOracleError::AssetNotFound(s) => format!("Asset {} not found.", s),
        }
    }
}

impl Into<StdError> for CommonOracleError {
    fn into(self) -> StdError {
        StdError::generic_err(self.to_string())
    }
}
