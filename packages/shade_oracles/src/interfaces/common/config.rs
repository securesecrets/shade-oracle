use super::*;

#[cw_serde]
pub struct CommonConfig {
    pub router: Contract,
    pub enabled: bool,
}

#[cw_serde]
pub struct CommonConfigResponse {
    pub config: CommonConfig,
    pub supported_keys: Vec<String>,
}

#[cfg(feature = "core")]
pub use state::*;
#[cfg(feature = "core")]
mod state {
    use shade_protocol::admin::{QueryMsg, ValidateAdminPermissionResponse};
    use ssp::Item;

    use crate::{
        error::CommonOracleError,
        interfaces::router::msg::{
            ConfigResponse as RouterConfigResponse, QueryMsg as RouterQueryMsg,
        },
        querier::{require_admin, require_admin_or_bot, require_bot},
    };

    use super::*;

    impl CommonConfig {
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
            let keys;
            if let Some(mut current_keys) = Self::SUPPORTED_KEYS.may_load(storage)? {
                if !current_keys.contains(key) {
                    current_keys.push(key.to_string());
                }
                keys = current_keys;
            } else {
                keys = vec![key.to_string()];
            }
            Self::SUPPORTED_KEYS.save(storage, &keys)?;
            Ok(())
        }

        pub fn init(
            api: &dyn Api,
            storage: &mut dyn Storage,
            router: RawContract,
        ) -> StdResult<Self> {
            let router = router.into_valid(api)?;
            Self::SUPPORTED_KEYS.save(storage, &vec![])?;
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
            require_admin(&self.router, querier, info.sender)
        }
        /// Queries the router to see if user has the oracles bot permission.
        pub fn require_bot(&self, querier: &QuerierWrapper, info: MessageInfo) -> StdResult<()> {
            require_bot(&self.router, querier, info.sender)
        }

        pub fn require_admin_or_bot(
            &self,
            querier: &QuerierWrapper,
            info: MessageInfo,
        ) -> StdResult<()> {
            require_admin_or_bot(&self.router, querier, info.sender)
        }

        pub fn require_permission(
            &self,
            querier: &QuerierWrapper,
            info: MessageInfo,
            permission: impl ToString,
        ) -> StdResult<()> {
            let get_admin_auth_req: RouterConfigResponse =
                RouterQueryMsg::GetConfig {}.query(querier, &self.router)?;
            let admin_auth = get_admin_auth_req.config.admin_auth;
            let admin_resp: ValidateAdminPermissionResponse = QueryMsg::ValidateAdminPermission {
                permission: permission.to_string(),
                user: info.sender.clone().into(),
            }
            .query(querier, &admin_auth)?;
            if admin_resp.has_permission {
                Ok(())
            } else {
                Err(StdError::generic_err(format!(
                    "User {} does not have permission {}",
                    info.sender,
                    permission.to_string()
                )))
            }
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
