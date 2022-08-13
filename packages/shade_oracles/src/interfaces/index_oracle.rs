use crate::common::{
    ConfigResponse, ConfigUpdates, InstantiateCommonConfig, PriceResponse, PricesResponse,
};
use shade_protocol::utils::Query;
use crate::BLOCK_SIZE;
use better_secret_math::U256;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal256, StdResult, Storage, Uint128};
use shade_protocol::{
    utils::generic_response::ResponseStatus,
    utils::{ExecuteCallback, InstantiateCallback},
};

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
    ModBasket {
        basket: Vec<(String, Decimal256)>,
    },
    UpdateConfig {
        updates: ConfigUpdates,
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

#[cw_serde]
/// The only symbol supported by the index oracle.
pub struct Symbol(pub String);

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

#[cfg(feature = "index")]
pub use state::*;
#[cfg(feature = "index")]
pub mod state {
    use super::*;
    use crate::storage::{Bincode2, GenericItemStorage, GenericMapStorage, Item, ItemStorage, Map};

    /// (weight, constant)
    pub type BtrBasketItem = (U256, U256);
    pub type BasketSymbol = String;

    pub struct BasketSymbols;
    pub struct BtrBasket;

    impl GenericItemStorage<Vec<BasketSymbol>> for BasketSymbols {
        const ITEM: Item<'static, Vec<BasketSymbol>> = Item::new("indexbasketsymbols");
    }

    #[cw_serde]
    pub struct Target(pub Uint128);

    impl ItemStorage for Target {
        const ITEM: Item<'static, Self> = Item::new("indextarget");
    }

    impl<'a> GenericMapStorage<'a, &'a str, BtrBasketItem, Bincode2> for BtrBasket {
        const MAP: Map<'static, &'a str, BtrBasketItem, Bincode2> = Map::new("indexbasketitems");
    }

    impl BtrBasket {
        pub fn load_basket(
            storage: &dyn Storage,
            symbols: &[String],
        ) -> StdResult<Vec<(BasketSymbol, U256, U256)>> {
            let mut basket: Vec<(BasketSymbol, U256, U256)> = vec![];
            for symbol in symbols {
                let item = BtrBasket::load(storage, symbol.as_str())?;
                basket.push((symbol.to_string(), item.0, item.1));
            }
            Ok(basket)
        }
    }

    impl ItemStorage for Symbol {
        const ITEM: Item<'static, Self> = Item::new("indexsymbol");
    }
}
