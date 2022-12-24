use super::*;

#[cw_serde]
pub struct CommonConfig {
    pub router: Contract,
    pub enabled: bool,
}

#[cw_serde]
pub struct CommonConfigResponse {
    pub config: CommonConfig,
}

#[cfg(feature = "core")]
pub use state::*;
#[cfg(feature = "core")]
mod state {
    use shade_protocol::admin::helpers::AdminPermissions;
    use ssp::Item;

    use crate::{error::CommonOracleError, querier::require_admin};

    use super::*;

    impl<'a> CommonConfig {
        pub const SUPPORTED_KEYS: Item<'static, Vec<String>> = Item::new("supported_keys");
    }

    impl CommonConfig {
        pub fn require_supported_key(storage: &dyn Storage, key: &String) -> StdResult<()> {
            if !Self::SUPPORTED_KEYS.load(storage)?.contains(key) {
                return Err(CommonOracleError::NotSupportedKey(key.to_string()).into());
            }
            Ok(())
        }

        pub fn add_supported_key(storage: &mut dyn Storage, key: &String) -> StdResult<()> {
            Self::SUPPORTED_KEYS.update(storage, |mut current_keys| -> StdResult<_> {
                if !current_keys.contains(&key) {
                    current_keys.push(key.to_string());
                }
                Ok(current_keys)
            })?;
            Ok(())
        }

        pub fn init(api: &dyn Api, router: RawContract) -> StdResult<Self> {
            let router = router.into_valid(api)?;
            Ok(CommonConfig {
                router,
                enabled: true,
            })
        }
        pub fn update_config(
            &mut self,
            api: &dyn Api,
            status: Option<bool>,
            router: Option<RawContract>,
        ) -> StdResult<()> {
            if let Some(router) = router {
                self.router = router.into_valid(api)?;
            }
            self.enabled = status.unwrap_or(self.enabled);
            Ok(())
        }
        /// Queries the router to see if user has the oracles admin permission.
        pub fn require_admin(&self, querier: &QuerierWrapper, info: MessageInfo) -> StdResult<()> {
            require_admin(
                &self.router,
                AdminPermissions::OraclesAdmin,
                querier,
                info.sender,
            )?;
            Ok(())
        }
        /// Queries the router to see if user has the oracles bot permission.
        pub fn require_bot(&self, querier: &QuerierWrapper, info: MessageInfo) -> StdResult<()> {
            require_admin(
                &self.router,
                AdminPermissions::OraclesPriceBot,
                querier,
                info.sender,
            )?;
            Ok(())
        }
        pub fn require_enabled(&self) -> StdResult<()> {
            if self.enabled {
                Ok(())
            } else {
                Err(StdError::generic_err(
                    "This oracle has been disabled. Only operation available is changing status.",
                ))
            }
        }
    }
}
