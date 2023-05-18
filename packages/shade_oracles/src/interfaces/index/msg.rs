use crate::{
    common::status::ContractStatus,
    impl_msg_callbacks,
    interfaces::common::{PriceResponse, PricesResponse},
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal256, Uint256, Uint64};
use shade_toolkit::{RawContract, Contract};

use super::{AssetSymbol, AssetWeights, Peg};

impl_msg_callbacks!();

pub type InitialBasketItem = (String, Decimal256);

#[cw_serde]
pub struct InstantiateMsg {
    pub router: RawContract,
    pub basket: Vec<InitialBasketItem>, //HashMap<String, Decimal256>,
    pub target: Uint256,
    pub symbol: String,
    pub when_stale: Uint64,
    pub deviation_threshold: Decimal256,
}

#[cw_serde]
pub enum ExecuteMsg {
    ComputeIndex {},
    Admin(AdminMsg),
}

#[cw_serde]
pub enum AdminMsg {
    UpdateStatus(ContractStatus),
    /// To remove assets that have been added, set the initial weight to be 0.
    /// All other weight values will cause the corresponding asset to be added or changed.
    /// Assets not included in the mod list will remain in the oracle until removed.
    ///
    /// Vec<(Symbol, Weight)> where Symbol is string and Weight is Decimal256
    ModBasket(Vec<InitialBasketItem>),
    UpdateConfig {
        symbol: Option<String>,
        router: Option<RawContract>,
        when_stale: Option<Uint64>,
        deviation_threshold: Option<Decimal256>,
    },
    UpdateTarget(Uint256),
    UnfreezePeg {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(PriceResponse)]
    GetPrice { key: String },
    #[returns(PricesResponse)]
    GetPrices { keys: Vec<String> },
    #[returns(IndexDataResponse)]
    GetIndexData {},
    #[returns(BasketResponse)]
    GetBasket {},
}

#[cw_serde]
pub struct IndexDataResponse {
    pub symbol: String,
    pub router: Contract,
    pub when_stale: Uint64,
    pub peg: Peg,
    pub basket: Vec<IndexAsset>,
}

#[cw_serde]
pub struct IndexAsset {
    pub symbol: AssetSymbol,
    pub weight: AssetWeights,
}

#[cw_serde]
pub struct BasketResponse {
    pub basket: Vec<(String, Decimal256, Decimal256)>,
}
