use crate::BLOCK_SIZE;
use crate::{Contract, ResponseStatus};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, QuerierWrapper};
use cosmwasm_std::{Querier, StdResult};
use shade_protocol::utils::{Query, ExecuteCallback, InstantiateCallback};

#[cw_serde]
pub struct InstantiateMsg {}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateSymbolPrice {
        base_symbol: String,
        quote_symbol: String,
        rate: Uint128,
        last_updated: Option<u64>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum HandleAnswer {
    UpdateSymbolPrice { status: ResponseStatus },
}

#[cw_serde]
pub enum OracleQuery {
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

impl Query for OracleQuery {
    const BLOCK_SIZE: usize = 256;
}

pub fn reference_data(
    querier: &QuerierWrapper,
    base_symbol: String,
    quote_symbol: String,
    band: Contract,
) -> StdResult<ReferenceData> {
    OracleQuery::GetReferenceData {
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
    OracleQuery::GetReferenceDataBulk {
        base_symbols,
        quote_symbols,
    }
    .query(querier, &band)
}

pub mod proxy {
    use shade_admin::storage::Item;
    use shade_protocol::utils::{asset::{RawContract, Contract}, storage::plus::ItemStorage};
    use crate::common::{ConfigUpdates, InstantiateCommonConfig, CommonConfig};

    use super::*;
    // base_asset quoted in quote_asset, Ex: BTC (base) quoted in USD(quote)
    #[cw_serde]
    pub struct InstantiateMsg {
        pub config: InstantiateCommonConfig,
        pub quote_symbol: String,
    }

    #[cw_serde]
    pub struct ConfigResponse {
        pub config: CommonConfig,
        pub quote_symbol: String,
    }

    #[cfg(feature = "proxy-band")]
    #[cw_serde]
    pub struct QuoteSymbol(String);

    #[cfg(feature = "proxy-band")]
    impl ItemStorage for QuoteSymbol {
        const ITEM: Item<'static, Self> = Item::new("quotesymbol");
    }

    impl InstantiateCallback for InstantiateMsg {
        const BLOCK_SIZE: usize = BLOCK_SIZE;
    }
}
