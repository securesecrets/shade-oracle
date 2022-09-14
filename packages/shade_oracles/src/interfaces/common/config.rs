use super::*;

#[cw_serde]
/// Config object passed into the updating of an oracle's common config.
///
/// supported_keys - (keys which are allowed by oracle, if none listed, then oracle will support all keys).
///
/// router - oracle router
pub struct ConfigUpdates {
    pub supported_keys: Option<Vec<String>>,
    pub router: Option<RawContract>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: CommonConfig,
}

#[cw_serde]
/// Config object passed into the instantiation of an oracle.
///
/// supported_keys - (keys which are allowed by oracle, if none listed, then oracle will support all keys).
///
/// router - oracle router
///
/// enabled - can we use this oracle?
///
/// only_band - will this oracle go directly to band rather than through the router?
///
/// will be deprecated
pub struct InstantiateCommonConfig {
    pub supported_keys: Option<Vec<String>>,
    pub router: RawContract,
}

impl InstantiateCommonConfig {
    pub fn new(supported_keys: Option<Vec<String>>, router: RawContract) -> Self {
        InstantiateCommonConfig {
            supported_keys,
            router,
        }
    }
    pub fn into_valid(self, api: &dyn Api) -> StdResult<CommonConfig> {
        Ok(CommonConfig {
            supported_keys: self.supported_keys.unwrap_or_default(),
            router: self.router.into_valid(api)?,
        })
    }
}

/// Config object stored in all oracles.
///
/// supported_keys - (keys which are allowed by oracle, if none listed, then oracle will support all keys).
///
/// router - oracle router
///
/// enabled - can we use this oracle?
///
/// only_band - will this oracle go directly to band rather than through the router?
#[cw_serde]
pub struct CommonConfig {
    pub supported_keys: Vec<String>,
    pub router: Contract,
}

#[cfg(feature = "core")]
pub use state::*;
#[cfg(feature = "core")]
mod state {
    use super::*;
    use crate::ssp::{Item, ItemStorage};

    impl ItemStorage for CommonConfig {
        const ITEM: Item<'static, Self> = Item::new("commonconfig");
    }
}
