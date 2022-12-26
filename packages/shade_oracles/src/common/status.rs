use std::error::Error;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, StdResult, Storage};
use secret_storage_plus::{Item, ItemStorage};

#[cw_serde]
/// Normal - all operations allowed except migrations
///
/// Deprecated - only time migrations are allowed, other permitted operations TBD by target
///
/// Frozen - only status change allowed
pub enum ContractStatus {
    Normal,
    Deprecated,
    Frozen,
}

impl Default for ContractStatus {
    fn default() -> Self {
        ContractStatus::Normal
    }
}

pub trait GlobalStatus<T: Error + Into<StdError>> {
    fn normal_err() -> T;
    fn deprecated_err() -> T;
    fn frozen_err() -> T;
    fn not_found() -> T;
    fn require_can_run(
        storage: &dyn Storage,
        when_normal: bool,
        when_deprecated: bool,
        when_frozen: bool,
    ) -> StdResult<()>
where {
        match ContractStatus::load(storage) {
            Ok(status) => {
                status.require_can_run(
                    when_normal,
                    when_deprecated,
                    when_frozen,
                    Self::normal_err().into(),
                    Self::deprecated_err().into(),
                    Self::frozen_err().into(),
                )?;
                Ok(())
            }
            Err(_) => Err(Self::not_found().into()),
        }
    }
    fn update_status(
        storage: &mut dyn Storage,
        new_status: ContractStatus,
    ) -> StdResult<ContractStatus> {
        new_status.save(storage)?;
        Ok(new_status)
    }
    fn init_status(storage: &mut dyn Storage) -> StdResult<ContractStatus> {
        let status = ContractStatus::Normal;
        status.save(storage)?;
        Ok(status)
    }
    fn load_status(storage: &dyn Storage) -> StdResult<ContractStatus> {
        ContractStatus::load(storage)
    }
}

impl ContractStatus {
    pub fn is_deprecated(&self) -> bool {
        self.eq(&ContractStatus::Deprecated)
    }
    pub fn is_frozen(&self) -> bool {
        self.eq(&ContractStatus::Frozen)
    }
    pub fn is_normal(&self) -> bool {
        self.eq(&ContractStatus::Normal)
    }
    /// Throws an error if some function cannot be run under the following conditions.
    pub fn require_can_run(
        &self,
        when_normal: bool,
        when_deprecated: bool,
        when_frozen: bool,
        normal_err: StdError,
        deprecated_err: StdError,
        frozen_err: StdError,
    ) -> StdResult<()> {
        if self.is_normal() && !when_normal {
            return Err(normal_err);
        }
        if self.is_deprecated() && !when_deprecated {
            return Err(deprecated_err);
        }
        if self.is_frozen() && !when_frozen {
            return Err(frozen_err);
        }
        Ok(())
    }
}

impl ItemStorage for ContractStatus {
    const ITEM: Item<'static, Self> = Item::new("globalcontractstatus");
}

#[macro_export(local_inner_macros)]
macro_rules! impl_global_status {
    ($struct:ident, $err:ident) => {
        impl $crate::common::status::GlobalStatus<$err> for $struct {
            fn normal_err() -> $err {
                $err::Normal
            }

            fn deprecated_err() -> $err {
                $err::Deprecated
            }

            fn frozen_err() -> $err {
                $err::Frozen
            }

            fn not_found() -> $err {
                $err::Std(cosmwasm_std::StdError::NotFound {
                    kind: "global status".to_string(),
                })
            }
        }
    };
}
