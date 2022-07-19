use crate::BLOCK_SIZE;
use shade_protocol::{
    Contract,
    utils::generic_response::ResponseStatus,
    utils::{InstantiateCallback, ExecuteCallback, Query},
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, QuerierWrapper};
use cosmwasm_std::{StdResult};

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
pub enum QueryMsg {
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

pub fn reference_data_bulk(
    querier: &QuerierWrapper,
    base_symbols: Vec<String>,
    quote_symbols: Vec<String>,
    band: &Contract,
) -> StdResult<Vec<ReferenceData>> {
    QueryMsg::GetReferenceDataBulk {
        base_symbols,
        quote_symbols,
    }
    .query(querier, band)
}

pub mod proxy {
    use shade_protocol::secret_storage_plus::Item;
    use shade_protocol::utils::{storage::plus::ItemStorage, asset::RawContract};
    use crate::common::{InstantiateCommonConfig, CommonConfig};

    use super::*;
    // base_asset quoted in quote_asset, Ex: BTC (base) quoted in USD(quote)
    #[cw_serde]
    pub struct InstantiateMsg {
        pub config: InstantiateCommonConfig,
        pub band: RawContract,
        pub quote_symbol: String,
    }

    #[cw_serde]
    pub struct ConfigResponse {
        pub config: CommonConfig,
        pub quote_symbol: String,
    }

    #[cw_serde]
    pub struct QuoteSymbol(pub String);

    #[cfg(feature = "proxy-band")]
    impl ItemStorage for QuoteSymbol {
        const ITEM: Item<'static, Self> = Item::new("quotesymbol");
    }

    impl InstantiateCallback for InstantiateMsg {
        const BLOCK_SIZE: usize = BLOCK_SIZE;
    }
}
