use super::{GasLog, TestableContract};
use crate::constants::*;
use secretcli::{cli_types::NetContract, secretcli::query};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use shade_oracles::{
    common::{PriceResponse, PricesResponse},
    core::{Contract, Decimal256, RawContract, Uint128, Uint64},
    interfaces::{
        band, common as common_oracles,
        index::{self, msg::AdminMsg as IndexAdminMsg},
        lp as lp_oracle,
        router::{
            self, msg as Router,
            registry::{RegistryOperation, UpdateConfig},
        },
        staking_derivative,
    },
};

#[derive(Serialize, Deserialize)]
pub struct OracleRouterContract {
    pub info: NetContract,
}

impl TestableContract for OracleRouterContract {
    fn get_info(&self) -> &NetContract {
        &self.info
    }
    fn get_file() -> &'static str {
        ORACLE_ROUTER_FILE
    }
}

impl OracleRouterContract {
    pub fn new(
        msg: &Router::InstantiateMsg,
        account_key: Option<&str>,
        name: Option<&str>,
        label: Option<&str>,
    ) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name, label)?;
        Ok(OracleRouterContract { info })
    }

    pub fn query_config(&self) -> Result<Router::ConfigResponse> {
        query(self.get_info(), Router::QueryMsg::GetConfig {}, None)
    }

    pub fn query_price(&self, key: String) -> Result<PriceResponse> {
        query(self.get_info(), Router::QueryMsg::GetPrice { key }, None)
    }

    pub fn query_prices(&self, keys: Vec<String>) -> Result<PricesResponse> {
        query(self.get_info(), Router::QueryMsg::GetPrices { keys }, None)
    }

    pub fn query_oracle(&self, key: String) -> Result<Router::OracleResponse> {
        query(self.get_info(), Router::QueryMsg::GetOracle { key }, None)
    }

    pub fn update_config(&self, config: UpdateConfig, sender_key: Option<&str>) -> Result<GasLog> {
        let msg = Router::ExecuteMsg::UpdateConfig { config };
        self.wrap_execute(&msg, sender_key)
    }

    pub fn update_registry(
        &self,
        operation: RegistryOperation,
        sender_key: Option<&str>,
    ) -> Result<GasLog> {
        let msg = Router::ExecuteMsg::UpdateRegistry { operation };
        self.wrap_execute(&msg, sender_key)
    }

    pub fn batch_update_registry(
        &self,
        operations: Vec<RegistryOperation>,
        sender_key: Option<&str>,
    ) -> Result<GasLog> {
        let msg = Router::ExecuteMsg::BatchUpdateRegistry { operations };
        self.wrap_execute(&msg, sender_key)
    }

    pub fn update_oracle(
        &self,
        sender_key: &str,
        symbol: &str,
        new_oracle: Contract,
    ) -> Result<()> {
        println!("Updating oracle at {}.", symbol);
        match self.query_oracle(symbol.to_string()) {
            Ok(_) => {
                self.update_registry(
                    RegistryOperation::Replace {
                        oracle: new_oracle,
                        key: symbol.to_string(),
                    },
                    Some(sender_key),
                )?;
            }
            Err(_) => {
                self.update_registry(
                    RegistryOperation::Add {
                        oracle: new_oracle,
                        key: symbol.to_string(),
                    },
                    Some(sender_key),
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct BandContract {
    pub info: NetContract,
}

impl TestableContract for BandContract {
    fn get_info(&self) -> &NetContract {
        &self.info
    }
    fn get_file() -> &'static str {
        MOCK_BAND_FILE
    }
}

#[derive(Serialize, Deserialize)]
pub struct SiennaMarketOracleContract {
    pub info: NetContract,
}
impl OracleContract for SiennaMarketOracleContract {}
impl TestableContract for SiennaMarketOracleContract {
    fn get_info(&self) -> &NetContract {
        &self.info
    }
    fn get_file() -> &'static str {
        SIENNA_MARKET_ORACLE_FILE
    }
}

impl SiennaMarketOracleContract {
    pub fn new(
        msg: &shade_oracles::interfaces::lp::market::InstantiateMsg,
        account_key: Option<&str>,
        name: Option<&str>,
        label: Option<&str>,
    ) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name, label)?;
        Ok(SiennaMarketOracleContract { info })
    }
}

#[derive(Serialize, Deserialize)]
pub struct SiennaswapSpotLpOracleContract {
    pub info: NetContract,
}

impl OracleContract for SiennaswapSpotLpOracleContract {}
impl TestableContract for SiennaswapSpotLpOracleContract {
    fn get_info(&self) -> &NetContract {
        &self.info
    }
    fn get_file() -> &'static str {
        SIENNASWAP_LP_SPOT_ORACLE_FILE
    }
}

#[derive(Serialize, Deserialize)]
pub struct ShadeStakingDerivativeOracleContract {
    pub info: NetContract,
}

impl OracleContract for ShadeStakingDerivativeOracleContract {}
impl TestableContract for ShadeStakingDerivativeOracleContract {
    fn get_info(&self) -> &NetContract {
        &self.info
    }
    fn get_file() -> &'static str {
        SHADE_STAKING_DERIVATIVE_ORACLE_FILE
    }
}

#[derive(Serialize, Deserialize)]
pub struct EarnV1OracleContract {
    pub info: NetContract,
}

impl OracleContract for EarnV1OracleContract {}
impl TestableContract for EarnV1OracleContract {
    fn get_info(&self) -> &NetContract {
        &self.info
    }
    fn get_file() -> &'static str {
        EARN_V1_ORACLE_FILE
    }
}

pub trait OracleContract
where
    Self: TestableContract,
{
    fn query_price(&self, key: String) -> Result<common_oracles::OraclePrice> {
        query(
            self.get_info(),
            shade_oracles::common::OracleQuery::GetPrice { key },
            None,
        )
    }
    fn query_config<Response: serde::de::DeserializeOwned>(&self) -> Result<Response> {
        query(
            self.get_info(),
            shade_oracles::common::OracleQuery::GetConfig {},
            None,
        )
    }
}

impl BandContract {
    pub fn new(
        msg: &band::InstantiateMsg,
        account_key: Option<&str>,
        name: Option<&str>,
        label: Option<&str>,
    ) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name, label)?;
        Ok(BandContract { info })
    }

    pub fn update_band_pair(
        &self,
        pair: (&str, &str),
        rate: Uint128,
        last_updated: Option<u64>,
        account_key: Option<&str>,
    ) -> Result<GasLog> {
        let msg = band::ExecuteMsg::UpdateSymbolPrice {
            base_symbol: pair.0.to_string(),
            quote_symbol: pair.1.to_string(),
            rate,
            last_updated,
        };
        self.wrap_execute(&msg, account_key)
    }
}

impl SiennaswapSpotLpOracleContract {
    pub fn new(
        msg: &lp_oracle::siennaswap::InstantiateMsg,
        account_key: Option<&str>,
        name: Option<&str>,
        label: Option<&str>,
    ) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name, label)?;
        Ok(SiennaswapSpotLpOracleContract { info })
    }
}

impl ShadeStakingDerivativeOracleContract {
    pub fn new(
        msg: &staking_derivative::shade::InstantiateMsg,
        account_key: Option<&str>,
        name: Option<&str>,
        label: Option<&str>,
    ) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name, label)?;
        Ok(ShadeStakingDerivativeOracleContract { info })
    }
}

#[derive(Serialize, Deserialize)]
pub struct IndexOracleContract {
    pub info: NetContract,
}

impl OracleContract for IndexOracleContract {}
impl TestableContract for IndexOracleContract {
    fn get_info(&self) -> &NetContract {
        &self.info
    }
    fn get_file() -> &'static str {
        INDEX_ORACLE_FILE
    }
}

impl IndexOracleContract {
    pub fn new(
        msg: &index::msg::InstantiateMsg,
        account_key: Option<&str>,
        name: Option<&str>,
        label: Option<&str>,
    ) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name, label)?;
        Ok(IndexOracleContract { info })
    }

    pub fn query_index_data(&self, key: String) -> Result<index::msg::IndexDataResponse> {
        query(self.get_info(), index::msg::QueryMsg::GetIndexData {}, None)
    }

    pub fn query_basket(&self) -> Result<index::msg::BasketResponse> {
        query(self.get_info(), index::msg::QueryMsg::GetBasket {}, None)
    }

    pub fn update_config(
        &self,
        router: Option<RawContract>,
        symbol: Option<String>,
        when_stale: Option<Uint64>,
        sender_key: Option<&str>,
    ) -> Result<GasLog> {
        let msg = index::msg::ExecuteMsg::Admin(IndexAdminMsg::UpdateConfig {
            symbol,
            router,
            when_stale,
        });
        self.wrap_execute(&msg, sender_key)
    }

    pub fn mod_basket(
        &self,
        basket: Vec<(String, Decimal256)>,
        sender_key: Option<&str>,
    ) -> Result<GasLog> {
        let msg = index::msg::ExecuteMsg::Admin(IndexAdminMsg::ModBasket { basket });
        self.wrap_execute(&msg, sender_key)
    }
}
