use core::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Api, CanonicalAddr, Extern, Addr, Querier, StdResult, Storage, Uint128};


use crate::common::Contract;

#[cw_serde]
pub struct Asset {
    pub info: AssetInfo,
    pub amount: Uint128,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.info)
    }
}

impl Asset {
    pub fn is_native_token(&self) -> bool {
        self.info.is_native_token()
    }

    pub fn to_raw(
        &self,
        deps: Deps,
    ) -> StdResult<AssetRaw> {
        Ok(AssetRaw {
            info: match &self.info {
                AssetInfo::NativeToken { denom } => AssetInfoRaw::NativeToken {
                    denom: denom.to_string(),
                },
                AssetInfo::Token {
                    contract_addr,
                    token_code_hash,
                    viewing_key,
                } => AssetInfoRaw::Token {
                    contract_addr: deps
                        .api
                        .canonical_address(&Addr::from(contract_addr.as_str()))?,
                    token_code_hash: token_code_hash.clone(),
                    viewing_key: viewing_key.clone(),
                },
            },
            amount: self.amount,
        })
    }
}

#[cw_serde]
pub enum AssetInfo {
    Token {
        contract_addr: String,
        token_code_hash: String,
        viewing_key: String,
    },
    NativeToken {
        denom: String,
    },
}

impl fmt::Display for AssetInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetInfo::NativeToken { denom } => write!(f, "{}", denom),
            AssetInfo::Token { contract_addr, .. } => write!(f, "{}", contract_addr),
        }
    }
}

impl AssetInfo {
    pub fn to_raw(
        &self,
        deps: Deps,
    ) -> StdResult<AssetInfoRaw> {
        match self {
            AssetInfo::NativeToken { denom } => Ok(AssetInfoRaw::NativeToken {
                denom: denom.to_string(),
            }),
            AssetInfo::Token {
                contract_addr,
                viewing_key,
                token_code_hash,
            } => Ok(AssetInfoRaw::Token {
                contract_addr: deps
                    .api
                    .canonical_address(&Addr::from(contract_addr.as_str()))?,
                viewing_key: viewing_key.clone(),
                token_code_hash: token_code_hash.clone(),
            }),
        }
    }

    pub fn is_native_token(&self) -> bool {
        match self {
            AssetInfo::NativeToken { .. } => true,
            AssetInfo::Token { .. } => false,
        }
    }

    pub fn equal(&self, asset: &AssetInfo) -> bool {
        match self {
            AssetInfo::Token { contract_addr, .. } => {
                let self_contract_addr = contract_addr;
                match asset {
                    AssetInfo::Token { contract_addr, .. } => self_contract_addr == contract_addr,
                    AssetInfo::NativeToken { .. } => false,
                }
            }
            AssetInfo::NativeToken { denom, .. } => {
                let self_denom = denom;
                match asset {
                    AssetInfo::Token { .. } => false,
                    AssetInfo::NativeToken { denom, .. } => self_denom == denom,
                }
            }
        }
    }
}

#[cw_serde]
pub struct AssetRaw {
    pub info: AssetInfoRaw,
    pub amount: Uint128,
}

impl AssetRaw {
    pub fn to_normal(
        &self,
        deps: Deps,
    ) -> StdResult<Asset> {
        Ok(Asset {
            info: match &self.info {
                AssetInfoRaw::NativeToken { denom } => AssetInfo::NativeToken {
                    denom: denom.to_string(),
                },
                AssetInfoRaw::Token {
                    contract_addr,
                    viewing_key,
                    token_code_hash,
                } => AssetInfo::Token {
                    contract_addr: deps.api.human_address(contract_addr)?.to_string(),
                    viewing_key: viewing_key.clone(),
                    token_code_hash: token_code_hash.clone(),
                },
            },
            amount: self.amount,
        })
    }
}

#[cw_serde]
pub enum AssetInfoRaw {
    Token {
        contract_addr: CanonicalAddr,
        token_code_hash: String,
        viewing_key: String,
    },
    NativeToken {
        denom: String,
    },
}

impl AssetInfoRaw {
    pub fn to_normal(
        &self,
        deps: Deps,
    ) -> StdResult<AssetInfo> {
        match self {
            AssetInfoRaw::NativeToken { denom } => Ok(AssetInfo::NativeToken {
                denom: denom.to_string(),
            }),
            AssetInfoRaw::Token {
                contract_addr,
                viewing_key,
                token_code_hash,
            } => Ok(AssetInfo::Token {
                contract_addr: deps.api.human_address(contract_addr)?.to_string(),
                viewing_key: viewing_key.clone(),
                token_code_hash: token_code_hash.clone(),
            }),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            AssetInfoRaw::NativeToken { denom } => denom.as_bytes(),
            AssetInfoRaw::Token { contract_addr, .. } => contract_addr.as_slice(),
        }
    }

    pub fn equal(&self, asset: &AssetInfoRaw) -> bool {
        match self {
            AssetInfoRaw::Token { contract_addr, .. } => {
                let self_contract_addr = contract_addr;
                match asset {
                    AssetInfoRaw::Token { contract_addr, .. } => {
                        self_contract_addr == contract_addr
                    }
                    AssetInfoRaw::NativeToken { .. } => false,
                }
            }
            AssetInfoRaw::NativeToken { denom, .. } => {
                let self_denom = denom;
                match asset {
                    AssetInfoRaw::Token { .. } => false,
                    AssetInfoRaw::NativeToken { denom, .. } => self_denom == denom,
                }
            }
        }
    }
}

#[cw_serde]
pub struct SecretSwapPairInfo {
    pub asset_infos: [AssetInfo; 2],
    pub contract_addr: Addr,
    pub liquidity_token: Addr,
    pub token_code_hash: String,
    pub asset0_volume: Uint128,
    pub asset1_volume: Uint128,
    pub factory: Contract,
}

#[cw_serde]
pub struct SecretSwapPoolResponse {
    pub assets: [Asset; 2],
    pub total_share: Uint128,
}

#[cw_serde]
pub enum SecretSwapPairQueryMsg {
    Pair {},
    Pool {},
}
