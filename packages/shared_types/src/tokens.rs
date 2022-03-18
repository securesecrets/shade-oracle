use crate::{
    asset::{CanonicalContract, Contract},
    scrt::{Api, Extern, Querier, StdError, Storage, Uint128},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RawToken {
    pub token: CanonicalContract,
    pub val: Uint128,
}

impl RawToken {
    pub fn as_human<S: Storage, A: Api, Q: Querier>(
        self,
        deps: &Extern<S, A, Q>,
    ) -> Result<HumanToken, StdError> {
        Ok(HumanToken {
            token: self.token.as_human(&deps.api)?,
            val: self.val,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HumanToken {
    pub token: Contract,
    pub val: Uint128,
}

impl HumanToken {
    pub fn as_canonical<S: Storage, A: Api, Q: Querier>(
        self,
        deps: &Extern<S, A, Q>,
    ) -> Result<RawToken, StdError> {
        Ok(RawToken {
            token: self.token.as_canonical(&deps.api)?,
            val: self.val,
        })
    }
}
