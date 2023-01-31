use crate::BLOCK_SIZE;
use better_secret_math::U256;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{QuerierWrapper, Timestamp, Uint128};
use cosmwasm_std::{StdError, StdResult};
use shade_protocol::admin::helpers::{validate_admin, AdminPermissions};
use shade_protocol::utils::asset::RawContract;
use shade_protocol::{
    utils::generic_response::ResponseStatus,
    utils::{ExecuteCallback, InstantiateCallback, Query},
    Contract,
};

use super::common::{PriceResponse, PricesResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_prices: Vec<(String, String, Uint128)>,
    pub admin_auth: RawContract,
    pub quote_symbol: Option<String>,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub struct Config {
    pub admin_auth: Contract,
    pub quote_symbol: String,
    pub enabled: bool,
}

impl Config {
    pub fn require_admin(
        &self,
        querier: &QuerierWrapper,
        user: impl Into<String> + Clone,
    ) -> StdResult<()> {
        validate_admin(
            querier,
            AdminPermissions::OraclesAdmin,
            user,
            &self.admin_auth,
        )
    }

    pub fn require_admin_or_bot(
        &self,
        querier: &QuerierWrapper,
        user: impl Into<String> + Clone,
    ) -> StdResult<()> {
        let admin_auth = &self.admin_auth;
        let is_admin = validate_admin(
            querier,
            AdminPermissions::OraclesAdmin,
            user.clone(),
            admin_auth,
        );
        let is_bot = validate_admin(querier, AdminPermissions::OraclesPriceBot, user, admin_auth);
        if is_admin.is_err() && is_bot.is_err() {
            Err(StdError::generic_err("User is not an admin or bot."))
        } else {
            Ok(())
        }
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    SetStatus(bool),
    UpdateConfig {
        admin_auth: Option<RawContract>,
        quote_symbol: Option<String>,
    },
    SetPrice(MockPrice),
    SetPrices(Vec<MockPrice>),
}

#[cw_serde]
pub struct MockPrice {
    pub base_symbol: String,
    pub quote_symbol: String,
    pub rate: Uint128,
    pub last_updated: Option<u64>,
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum ExecuteAnswer {
    SetPrice { status: ResponseStatus },
    SetPrices { status: ResponseStatus },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ReferenceData)]
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
    #[returns(ReferenceDataBulk)]
    GetReferenceDataBulk {
        base_symbols: Vec<String>,
        quote_symbols: Vec<String>,
    },
    #[returns(PriceResponse)]
    GetPrice { key: String },
    #[returns(PricesResponse)]
    GetPrices { keys: Vec<String> },
    #[returns(ConfigResponse)]
    GetConfig {},
}

pub type ConfigResponse = Config;

#[derive(Default)]
#[cw_serde]
pub struct ReferenceData {
    pub rate: Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

impl ReferenceData {
    pub fn new(rate: Uint128, last_updated_base: u64, last_updated_quote: u64) -> Self {
        ReferenceData {
            rate,
            last_updated_base,
            last_updated_quote,
        }
    }

    pub fn now(rate: Uint128, time: &Timestamp) -> Self {
        ReferenceData {
            rate,
            last_updated_base: time.seconds(),
            last_updated_quote: time.seconds(),
        }
    }
}

impl From<BtrReferenceData> for ReferenceData {
    fn from(r: BtrReferenceData) -> Self {
        ReferenceData {
            rate: r.rate.into(),
            last_updated_base: r.last_updated_base,
            last_updated_quote: r.last_updated_quote,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct BtrReferenceData {
    pub rate: U256,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

impl From<ReferenceData> for BtrReferenceData {
    fn from(r: ReferenceData) -> Self {
        BtrReferenceData {
            rate: r.rate.into(),
            last_updated_base: r.last_updated_base,
            last_updated_quote: r.last_updated_quote,
        }
    }
}

#[cw_serde]
pub struct ReferenceDataBulk {
    pub data: Vec<ReferenceData>,
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

pub fn reference_data(
    querier: &QuerierWrapper,
    base_symbol: String,
    quote_symbol: String,
    band: &Contract,
) -> StdResult<ReferenceData> {
    QueryMsg::GetReferenceData {
        base_symbol,
        quote_symbol,
    }
    .query(querier, band)
}

pub fn reference_data_bulk<I>(
    querier: &QuerierWrapper,
    base_symbols: I,
    quote_symbols: I,
    band: &Contract,
) -> StdResult<Vec<ReferenceData>>
where
    I: IntoIterator<Item = String>,
{
    QueryMsg::GetReferenceDataBulk {
        base_symbols: base_symbols.into_iter().collect(),
        quote_symbols: quote_symbols.into_iter().collect(),
    }
    .query(querier, band)
}
