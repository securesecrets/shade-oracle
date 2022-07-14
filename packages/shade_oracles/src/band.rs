use crate::common::{Contract, ResponseStatus};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, QuerierWrapper};
use cosmwasm_std::{Querier, StdResult};
use shade_protocol::utils::Query;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateSymbolPrice {
        base_symbol: String,
        quote_symbol: String,
        rate: Uint128,
        last_updated: Option<u64>,
    },
}

#[cw_serde]
pub enum HandleAnswer {
    UpdateSymbolPrice { status: ResponseStatus },
}

#[cw_serde]
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

#[cw_serde]
pub struct ReferenceData {
    pub rate: Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

#[cw_serde]
pub struct ReferenceDataBulk {
    pub data: Vec<ReferenceData>,
}

impl Query for BandQuery {
    const BLOCK_SIZE: usize = 256;
}

pub fn reference_data(
    querier: &QuerierWrapper,
    base_symbol: String,
    quote_symbol: String,
    band: Contract,
) -> StdResult<ReferenceData> {
    BandQuery::GetReferenceData {
        base_symbol,
        quote_symbol,
    }
    .query(querier, &band)
}

pub fn reference_data_bulk(
    querier: &QuerierWrapper,
    base_symbols: Vec<String>,
    quote_symbols: Vec<String>,
    band: Contract,
) -> StdResult<Vec<ReferenceData>> {
    BandQuery::GetReferenceDataBulk {
        base_symbols,
        quote_symbols,
    }
    .query(querier, &band)
}

pub mod proxy {
    use shade_protocol::utils::asset::{UnvalidatedContract, Contract};
    use super::*;
    // base_asset quoted in quote_asset, Ex: BTC (base) quoted in USD(quote)
    #[cw_serde]
    pub struct InstantiateMsg {
        pub admin_auth: Contract,
        pub band: Contract,
        pub quote_symbol: String,
    }

    #[cw_serde]
    pub struct Config {
        pub admin_auth: Contract,
        pub band: Contract,
        pub quote_symbol: String,
        pub enabled: bool,
    }

    /// Every ExecuteMsg for each specific oracle type should include this
    #[cw_serde]
    pub enum ExecuteMsg {
        UpdateConfig {
            enabled: Option<bool>,
            admin_auth: Option<Contract>,
            band: Option<Contract>,
            quote_symbol: Option<String>,
        },
    }

    #[cw_serde]
    pub enum HandleAnswer {
        UpdateConfig { status: ResponseStatus },
    }
}
