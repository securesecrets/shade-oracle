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
    use std::collections::HashSet;

    use secret_borsh_storage::BorshItem;
    use shade_protocol::admin::{QueryMsg, ValidateAdminPermissionResponse};

    use crate::{
        error::CommonOracleError,
        interfaces::router::msg::{
            ConfigResponse as RouterConfigResponse, QueryMsg as RouterQueryMsg,
        },
        querier::{query_price, require_admin, require_admin_or_bot, require_bot},
    };

    use super::*;

    impl CommonConfig {
        pub const SUPPORTED_KEYS: BorshItem<'static, HashSet<String>> =
            BorshItem::new("supported_keys");
    }

    impl CommonConfig {
        pub fn require_valid_router_symbol(
            &self,
            querier: &QuerierWrapper,
            symbol: &str,
        ) -> StdResult<()> {
            let resp = query_price(&self.router, querier, symbol.to_string());
            if resp.is_err() {
                Err(CommonOracleError::InvalidRouterSymbol(symbol.to_string()).into())
            } else {
                Ok(())
            }
        }

        pub fn require_supported_key(storage: &dyn Storage, key: &String) -> StdResult<()> {
            if !Self::SUPPORTED_KEYS.load(storage)?.contains(key) {
                return Err(CommonOracleError::NotSupportedKey(key.to_string()).into());
            }
            Ok(())
        }

        pub fn add_supported_key(storage: &mut dyn Storage, key: &String) -> StdResult<()> {
            Self::SUPPORTED_KEYS.update(storage, |mut keys| -> StdResult<_> {
                keys.insert(key.to_string());
                Ok(keys)
            })?;
            Ok(())
        }

        pub fn init(
            api: &dyn Api,
            storage: &mut dyn Storage,
            router: RawContract,
        ) -> StdResult<Self> {
            let router = router.into_valid(api)?;
            Self::SUPPORTED_KEYS.save(storage, &HashSet::new())?;
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
                Err(
                    CommonOracleError::UnauthorizedPermission(info.sender, permission.to_string())
                        .into(),
                )
            }
        }

        pub fn require_enabled(&self) -> StdResult<()> {
            if self.enabled {
                Ok(())
            } else {
                Err(CommonOracleError::DisabledOracle.into())
            }
        }

        pub fn get_resp(&self, storage: &dyn Storage) -> StdResult<CommonConfigResponse> {
            let supported_keys = Self::SUPPORTED_KEYS.load(storage)?;
            Ok(CommonConfigResponse {
                config: self.clone(),
                supported_keys: supported_keys.into_iter().collect(),
            })
        }
    }
}
