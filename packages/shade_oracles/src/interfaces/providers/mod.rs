use crate::BLOCK_SIZE;
use better_secret_math::U256;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Api, QuerierWrapper, Timestamp, Uint128, Uint256, Uint64};
use cosmwasm_std::{StdError, StdResult};
use shade_protocol::admin::helpers::{validate_admin, AdminPermissions};
use shade_protocol::utils::asset::RawContract;
use shade_protocol::{
    utils::generic_response::ResponseStatus,
    utils::{ExecuteCallback, InstantiateCallback, Query},
    Contract,
};

use self::mock::ConfigResponse;

use super::common::{PriceResponse, PricesResponse};
pub mod mock;

#[derive(Default)]
#[cw_serde]
pub struct ReferenceData {
    pub rate: Uint256,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

#[derive(Default)]
#[cw_serde]
pub struct OjoReferenceData {
    pub rate: Uint256,
    pub last_updated_base: Uint64,
    pub last_updated_quote: Uint64,
}

impl From<OjoReferenceData> for ReferenceData {
    fn from(r: OjoReferenceData) -> Self {
        ReferenceData {
            rate: r.rate,
            last_updated_base: r.last_updated_base.into(),
            last_updated_quote: r.last_updated_quote.into(),
        }
    }
}

impl ReferenceData {
    pub fn new(rate: Uint256, last_updated_base: u64, last_updated_quote: u64) -> Self {
        ReferenceData {
            rate,
            last_updated_base,
            last_updated_quote,
        }
    }

    pub fn now(rate: Uint256, time: &Timestamp) -> Self {
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
#[derive(QueryResponses)]
pub enum BandQueryMsg {
    #[returns(ReferenceData)]
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
    #[returns(Vec<ReferenceData>)]
    GetReferenceDataBulk {
        base_symbols: Vec<String>,
        quote_symbols: Vec<String>,
    },
    // Used for mocking
    #[returns(PriceResponse)]
    GetPrice { key: String },
    #[returns(PricesResponse)]
    GetPrices { keys: Vec<String> },
    #[returns(ConfigResponse)]
    GetConfig {},
}

impl Query for BandQueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum OjoQueryMsg {
    #[returns(OjoReferenceData)]
    GetReferenceData { symbol_pair: (String, String) },
    #[returns(Vec<OjoReferenceData>)]
    GetReferenceDataBulk { symbol_pairs: Vec<(String, String)> },
    #[returns(OjoReferenceData)]
    GetMedianReferenceData { symbol_pair: (String, String) },
    #[returns(Vec<OjoReferenceData>)]
    GetMedianReferenceDataBulk { symbol_pairs: Vec<(String, String)> },
    // Used for mocking
    #[returns(PriceResponse)]
    GetPrice { key: String },
    #[returns(PricesResponse)]
    GetPrices { keys: Vec<String> },
    #[returns(ConfigResponse)]
    GetConfig {},
}

impl Query for OjoQueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum RawProvider {
    Band(RawContract),
    Ojo(RawContract),
}

impl RawProvider {
    pub fn into_valid(self, api: &dyn Api) -> StdResult<Provider> {
        match self {
            RawProvider::Band(c) => Ok(Provider::Band(c.into_valid(api)?)),
            RawProvider::Ojo(c) => Ok(Provider::Ojo(c.into_valid(api)?)),
        }
    }
}

#[cw_serde]
pub enum Provider {
    Band(Contract),
    Ojo(Contract),
}

impl Provider {
    pub fn reference_data(
        &self,
        querier: &QuerierWrapper,
        symbol_pair: (&str, &str),
    ) -> StdResult<ReferenceData> {
        match self {
            Provider::Band(c) => BandQueryMsg::GetReferenceData {
                base_symbol: symbol_pair.0.to_string(),
                quote_symbol: symbol_pair.1.to_string(),
            }
            .query(querier, c),
            Provider::Ojo(c) => {
                let data: OjoReferenceData = OjoQueryMsg::GetReferenceData {
                    symbol_pair: (symbol_pair.0.to_string(), symbol_pair.1.to_string()),
                }
                .query(querier, c)?;
                Ok(data.into())
            }
        }
    }
    pub fn reference_data_bulk<I>(
        &self,
        querier: &QuerierWrapper,
        symbols: I,
    ) -> StdResult<Vec<ReferenceData>>
    where
        I: IntoIterator<Item = (String, String)>,
    {
        match self {
            Provider::Band(c) => {
                let mut base_symbols = vec![];
                let mut quote_symbols = vec![];
                for (base, quote) in symbols {
                    base_symbols.push(base);
                    quote_symbols.push(quote);
                }
                BandQueryMsg::GetReferenceDataBulk {
                    base_symbols,
                    quote_symbols,
                }
                .query(querier, c)
            }
            Provider::Ojo(c) => {
                let data: Vec<OjoReferenceData> = OjoQueryMsg::GetReferenceDataBulk {
                    symbol_pairs: symbols.into_iter().map(|s| (s.0, s.1)).collect(),
                }
                .query(querier, c)?;
                Ok(data.into_iter().map(|d| d.into()).collect())
            }
        }
    }
    pub fn median_reference_data(
        &self,
        querier: &QuerierWrapper,
        symbol_pair: (&str, &str),
    ) -> StdResult<ReferenceData> {
        match self {
            Provider::Band(_) => Err(ProviderError::Unimplemented.into()),
            Provider::Ojo(c) => {
                let data: OjoReferenceData = OjoQueryMsg::GetMedianReferenceData {
                    symbol_pair: (symbol_pair.0.to_string(), symbol_pair.1.to_string()),
                }
                .query(querier, c)?;
                Ok(data.into())
            }
        }
    }
    pub fn median_reference_data_bulk<I>(
        &self,
        querier: &QuerierWrapper,
        symbols: I,
    ) -> StdResult<Vec<ReferenceData>>
    where
        I: IntoIterator<Item = (String, String)>,
    {
        match self {
            Provider::Band(_) => Err(ProviderError::Unimplemented.into()),
            Provider::Ojo(c) => {
                let data: Vec<OjoReferenceData> = OjoQueryMsg::GetMedianReferenceDataBulk {
                    symbol_pairs: symbols.into_iter().map(|s| (s.0, s.1)).collect(),
                }
                .query(querier, c)?;
                Ok(data.into_iter().map(|d| d.into()).collect())
            }
        }
    }
}

#[cw_serde]
pub enum ProviderError {
    Unimplemented,
}

impl ToString for ProviderError {
    fn to_string(&self) -> String {
        match self {
            ProviderError::Unimplemented => "Unimplemented".to_string(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<StdError> for ProviderError {
    fn into(self) -> StdError {
        StdError::generic_err(self.to_string())
    }
}
