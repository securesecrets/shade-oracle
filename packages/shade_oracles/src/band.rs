use crate::common::{Contract, ResponseStatus};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{Querier, StdResult};
use schemars::JsonSchema;
use secret_toolkit::utils::Query;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateSymbolPrice {
        base_symbol: String,
        quote_symbol: String,
        rate: Uint128,
        last_updated: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateSymbolPrice { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BandQuery {
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
    GetReferenceDataBulk {
        base_symbols: Vec<String>,
        quote_symbols: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct ReferenceData {
    pub rate: Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReferenceDataBulk {
    pub data: Vec<ReferenceData>,
}

impl Query for BandQuery {
    const BLOCK_SIZE: usize = 256;
}

pub fn reference_data(
    querier: &impl Querier,
    base_symbol: String,
    quote_symbol: String,
    band: Contract,
) -> StdResult<ReferenceData> {
    BandQuery::GetReferenceData {
        base_symbol,
        quote_symbol,
    }
    .query(querier, band.code_hash, band.address)
}

pub fn reference_data_bulk(
    querier: &impl Querier,
    base_symbols: Vec<String>,
    quote_symbols: Vec<String>,
    band: Contract,
) -> StdResult<Vec<ReferenceData>> {
    BandQuery::GetReferenceDataBulk {
        base_symbols,
        quote_symbols,
    }
    .query(querier, band.code_hash, band.address)
}

pub mod proxy {
    use crate::common::Contract;

    use super::*;
    // base_asset quoted in quote_asset, Ex: BTC (base) quoted in USD(quote)
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub admin_auth: Contract,
        pub band: Contract,
        pub quote_symbol: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct Config {
        pub admin_auth: Contract,
        pub band: Contract,
        pub quote_symbol: String,
        pub enabled: bool,
    }

    /// Every HandleMsg for each specific oracle type should include this
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        UpdateConfig {
            enabled: Option<bool>,
            admin_auth: Option<Contract>,
            band: Option<Contract>,
            quote_symbol: Option<String>,
        },
    }

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleAnswer {
        UpdateConfig { status: ResponseStatus },
    }
}
