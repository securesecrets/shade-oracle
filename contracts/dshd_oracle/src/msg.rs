use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, Addr, QuerierWrapper, StdError, Storage, Uint128, Uint256};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use shade_oracles::better_secret_math::common::exp10;
use shade_oracles::core::{pad_query_result, validate_admin, AdminPermissions, ResponseStatus};
use shade_oracles::interfaces::common::{OraclePrice, OracleQuery, PriceResponse};
use shade_oracles::interfaces::providers::ReferenceData;
use shade_oracles::ssp::{Item, Map};
use shade_toolkit::{Contract, Query, RawContract, BLOCK_SIZE};

#[cw_serde]
pub struct Config {
    pub router: Contract,
    pub dshd: Contract,
    pub admin_auth: Contract,
    pub enabled: bool,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub router: RawContract,
    pub dshd: RawContract,
    pub admin_auth: RawContract,
}
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        router: Option<RawContract>,
        dshd: Option<RawContract>,
        admin_auth: Option<RawContract>,
        enabled: Option<bool>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetPrice { key: String },
    GetPrices { keys: Vec<String> },
    GetConfig {},
}
