use crate::{
    common::status::ContractStatus,
    impl_msg_callbacks,
    interfaces::common::{PriceResponse, PricesResponse},
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal256, Uint128, Uint64};
use shade_protocol::{utils::asset::RawContract, Contract};

use super::{AssetSymbol, AssetWeights, Target};

impl_msg_callbacks!();

pub type InitialBasketItem = (String, Decimal256);

#[cw_serde]
pub struct InstantiateMsg {
    pub router: RawContract,
    pub basket: Vec<InitialBasketItem>, //HashMap<String, Decimal256>,
    pub target: Uint128,
    pub symbol: String,
    pub when_stale: Uint64,
}

#[cw_serde]
pub enum ExecuteMsg {
    ComputeIndex {},
    Admin(AdminMsg),
}

#[cw_serde]
pub enum AdminMsg {
    UpdateStatus(ContractStatus),
    // Asset with weight 0 will be removed
    // all others are added or changed
    /// (symbol, weight)
    ModBasket(Vec<InitialBasketItem>),
    UpdateConfig {
        symbol: Option<String>,
        router: Option<RawContract>,
        when_stale: Option<Uint64>,
    },
    UpdateTarget(Uint128),
    Unfreeze {},
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
    pub target: Target,
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
