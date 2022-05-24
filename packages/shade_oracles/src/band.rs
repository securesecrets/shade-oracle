use crate::{
    common::ResponseStatus,
    scrt::{secret_toolkit::utils::Query, *},
};
use schemars::JsonSchema;
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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

pub mod proxy {
    use crate::common::Contract;

    use super::*;
    // base_asset quoted in quote_asset, Ex: BTC (base) quoted in USD(quote)
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub owner: HumanAddr,
        pub band: Contract,
        pub quote_symbol: String,
    }

    // We define a custom struct for each query response
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct ConfigResponse {
        pub owner: HumanAddr,
        pub band: Contract,
        pub quote_symbol: String,
        pub enabled: bool,
    }

    /// Every HandleMsg for each specific oracle type should include this
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        SetStatus { enabled: bool },
        UpdateConfig { owner: Option<HumanAddr>, band: Option<Contract>, quote_symbol: Option<String>, },
    }

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleAnswer {
        UpdateConfig { status: ResponseStatus }
    }
}
