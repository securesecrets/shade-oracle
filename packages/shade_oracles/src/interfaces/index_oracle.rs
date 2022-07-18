use crate::common::{InstantiateCommonConfig, ConfigUpdates};
use crate::{ResponseStatus, BLOCK_SIZE};
use crate::{InstantiateCallback, ExecuteCallback};
use crate::storage::{Item, ItemStorage};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use shade_protocol::utils::Query;

#[cw_serde]
/// Config doesn't need list of symbols, supported keys. or dependencies.
pub struct InstantiateMsg {
    pub config: InstantiateCommonConfig,
    pub basket: Vec<(String, Uint128)>, //HashMap<String, Uint128>,
    pub target: Uint128,
    pub symbol: String,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum ExecuteMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    ModBasket {
        basket: Vec<(String, Uint128)>,
    },
    UpdateConfig {
        updates: ConfigUpdates
    },
    UpdateTarget {
        new_target: Option<Uint128>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum HandleAnswer {
    ModBasket { status: ResponseStatus },
    UpdateConfig { status: ResponseStatus },
    UpdateTarget { status: ResponseStatus },
}

#[cw_serde]
pub enum QueryMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    GetPrice { key: String },
    GetPrices { keys: Vec<String> },
    GetConfig {},
    Basket {},
    GetTarget {},
    //Constants { },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
/// (symbol, weight, constant)
pub struct Basket(pub Vec<(String, Uint128, Uint128)>);

#[cfg(feature = "index")]
impl ItemStorage for Basket {
    const ITEM: Item<'static, Self> = Item::new("indexbasket");
}

#[cw_serde]
pub struct Target(pub Uint128);

#[cfg(feature = "index")]
impl ItemStorage for Target {
    const ITEM: Item<'static, Self> = Item::new("indextarget");
}

#[cw_serde]
/// The only symbol supported by the index oracle.
pub struct Symbol(pub String);

#[cfg(feature = "index")]
impl ItemStorage for Symbol {
    const ITEM: Item<'static, Self> = Item::new("indexsymbol");
}

#[cw_serde]
pub struct BasketResponse {
    pub basket: Vec<(String, Uint128, Uint128)>,
}

#[cw_serde]
pub struct TargetResponse {
    pub target: Uint128,
}