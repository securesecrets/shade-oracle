use crate::common::{
    ConfigResponse, ConfigUpdates, InstantiateCommonConfig, PriceResponse, PricesResponse,
};
use crate::core::{Query};
#[cfg(feature = "index")]
use crate::storage::{Item, ItemStorage, GenericItemStorage, Bincode2, GenericMapStorage, Map};
use crate::BLOCK_SIZE;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Decimal256, Deps, StdResult, Storage};
use shade_protocol::{
    utils::generic_response::ResponseStatus,
    utils::{ExecuteCallback, InstantiateCallback},
};
use better_secret_math::U256;

#[cw_serde]
/// Config doesn't need list of symbols, supported keys. or dependencies.
pub struct InstantiateMsg {
    pub config: InstantiateCommonConfig,
    pub basket: Vec<(String, Decimal256)>, //HashMap<String, Decimal256>,
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
    /// (symbol, weight)
    ModBasket { basket: Vec<(String, Decimal256)> },
    UpdateConfig { updates: ConfigUpdates },
    UpdateTarget { new_target: Option<Uint128> },
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
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    #[returns(PriceResponse)]
    GetPrice { key: String },
    #[returns(PricesResponse)]
    GetPrices { keys: Vec<String> },
    #[returns(ConfigResponse)]
    GetConfig {},
    #[returns(BasketResponse)]
    Basket {},
    #[returns(TargetResponse)]
    GetTarget {},
    //Constants { },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

/// (weight, constant)
pub type BtrBasketItem = (U256, U256);
pub type BasketSymbol = String;

pub struct BasketSymbols;
pub struct BtrBasket;

#[cfg(feature = "index")]
impl GenericItemStorage<Vec<BasketSymbol>> for BasketSymbols {
    const ITEM: Item<'static, Vec<BasketSymbol>> = Item::new("indexbasketsymbols");
}

#[cfg(feature = "index")]
impl<'a> GenericMapStorage<'a, &'a str, BtrBasketItem, Bincode2> for BtrBasket {
    const MAP: Map<'static, &'a str, BtrBasketItem, Bincode2> = Map::new("indexbasketitems");
}

impl BtrBasket {
    pub fn load_basket(storage: &dyn Storage, symbols: &[String]) -> StdResult<Vec<(BasketSymbol, U256, U256)>> {
        let mut basket: Vec<(BasketSymbol, U256, U256)> = vec![];
        for symbol in symbols {
            let item = BtrBasket::load(storage, symbol.as_str())?;
            basket.push((symbol.to_string(), item.0, item.1));
        }
        Ok(basket)
    }
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

/// (sym, weight, constant)
pub type BasketResponseItem = (String, Decimal256, Uint128);

#[cw_serde]
pub struct BasketResponse {
    pub basket: Vec<BasketResponseItem>,
}

#[cw_serde]
pub struct TargetResponse {
    pub target: Uint128,
}
