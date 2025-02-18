use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, Addr, Decimal, QuerierWrapper, StdError, Storage, Uint128, Uint256,
};
use cosmwasm_std::{
    to_binary, Binary, ContractInfo, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use shade_oracles::better_secret_math::common::exp10;
use shade_oracles::core::{pad_query_result, validate_admin, AdminPermissions, ResponseStatus};
use shade_oracles::interfaces::common::{OraclePrice, OracleQuery, PriceResponse};
use shade_oracles::interfaces::providers::ReferenceData;
use shade_oracles::ssp::{Item, Map};
use shade_toolkit::{Contract, Query, RawContract, BLOCK_SIZE};

#[cw_serde]
pub enum QueryMsg {
    GetVault { token: String },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub struct VaultState {
    pub token: ContractInfo,
    pub x_token: ContractInfo,
    pub decimals: u8,
    pub oracle_key: String,
    pub interest: Interest,
    pub loanable: Uint128,
    pub lent_amount: Uint128,
    pub lifetime_interest_paid: Uint128,
    pub lifetime_interest_owed: Uint128,
    pub interest_per_utoken: Decimal,
    pub last_interest_accrued: u64,
    pub max_supply: Uint128,
    pub max_borrow: Uint128,
    pub dao_interest_fee: Decimal,
    pub flash_loan_interest: Decimal,
    pub dao_flash_loan_interest_fee: Decimal,
    pub status: VaultStatus,
}

#[cw_serde]
pub struct Interest {
    pub base: Decimal,
    pub slope1: Decimal,
    pub slope2: Decimal,
    pub optimal_utilization: Decimal,
}

#[cw_serde]
pub struct VaultStatus {
    pub supply_enabled: bool,
    pub borrow_enabled: bool,
    pub repay_enabled: bool,
    pub liquidation_enabled: bool,
    pub interest_accrual_enabled: bool,
    pub flash_loan_enabled: bool,
}
