use crate::BLOCK_SIZE;
use better_secret_math::U256;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::StdResult;
use cosmwasm_std::{QuerierWrapper, Uint128};
use shade_protocol::{
    utils::generic_response::ResponseStatus,
    utils::{ExecuteCallback, InstantiateCallback, Query},
    Contract,
};

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
}

#[derive(Default)]
#[cw_serde]
pub struct ReferenceData {
    pub rate: Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

impl From<ReferenceData> for BtrReferenceData {
    fn from(r: ReferenceData) -> Self {
        BtrReferenceData {
            rate: U256::new(r.rate.u128()),
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

pub mod proxy {
    use crate::common::{CommonConfig, InstantiateCommonConfig};
    use shade_protocol::secret_storage_plus::Item;
    use shade_protocol::utils::{asset::RawContract, storage::plus::ItemStorage};

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
