use crate::constants::*;
use secretcli::{cli_types::NetContract, secretcli::query_contract};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use shade_oracles::{
    band, band::proxy as proxy_band_oracle, common as common_oracles, common::Contract,
    earn as earn_v1_oracle, lp as lp_oracle, router::{self, RegistryOperation}, index_oracle,
};
use cosmwasm_math_compat::Uint128;
use super::{GasLog, TestableContract};

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
        msg: &router::InitMsg,
        account_key: Option<&str>,
        name: Option<&str>,
    ) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name)?;
        Ok(OracleRouterContract { info })
    }

    pub fn query_config(&self) -> Result<router::Config> {
        query_contract(self.get_info(), router::QueryMsg::GetConfig { })
    }

    pub fn query_price(&self, key: String) -> Result<common_oracles::OraclePrice> {
        query_contract(self.get_info(), router::QueryMsg::GetPrice { key })
    }

    pub fn query_oracle(&self, key: String) -> Result<router::OracleResponse> {
        query_contract(self.get_info(), router::QueryMsg::GetOracle { key })
    }

    pub fn update_config(&self, admin_auth: Option<Contract>, default_oracle: Option<Contract>, sender_key: Option<&str>) -> Result<GasLog> {
        let msg = router::HandleMsg::UpdateConfig { admin_auth, default_oracle };
        self.wrap_handle(&msg, sender_key)
    }

    pub fn update_registry(
        &self,
        operation: router::RegistryOperation,
        sender_key: Option<&str>,
    ) -> Result<GasLog> {
        let msg = router::HandleMsg::UpdateRegistry { operation };
        self.wrap_handle(&msg, sender_key)
    }

    pub fn batch_update_registry(
        &self,
        operations: Vec<router::RegistryOperation>,
        sender_key: Option<&str>,
    ) -> Result<GasLog> {
        let msg = router::HandleMsg::BatchUpdateRegistry { operations };
        self.wrap_handle(&msg, sender_key)
    }

    pub fn update_oracle(
        &self,
        sender_key: &str, 
        symbol: &str, 
        new_oracle: Contract
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
            },
            Err(_) => {
                self.update_registry(
                    RegistryOperation::Add {
                        oracle: new_oracle,
                        key: symbol.to_string(),
                    },
                    Some(sender_key),
                )?;
            },
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
pub struct ProxyBandOracleContract {
    pub info: NetContract,
}

impl OracleContract for ProxyBandOracleContract {}
impl TestableContract for ProxyBandOracleContract {
    fn get_info(&self) -> &NetContract {
        &self.info
    }
    fn get_file() -> &'static str {
        PROXY_BAND_ORACLE_FILE
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
        msg: &shade_oracles::siennaswap_market_oracle::InitMsg,
        account_key: Option<&str>,
        name: Option<&str>,
    ) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name)?;
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
        query_contract(
            self.get_info(),
            shade_oracles::common::QueryMsg::GetPrice { key },
        )
    }
    fn query_config<Response: serde::de::DeserializeOwned>(&self) -> Result<Response> {
        query_contract(
            self.get_info(),
            shade_oracles::common::QueryMsg::GetConfig {},
        )
    }
}

impl BandContract {
    pub fn new(msg: &band::InitMsg, account_key: Option<&str>, name: Option<&str>) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name)?;
        Ok(BandContract { info })
    }

    pub fn update_band_pair(
        &self,
        pair: (&str, &str),
        rate: Uint128,
        last_updated: Option<u64>,
        account_key: Option<&str>,
    ) -> Result<GasLog> {
        let msg = shade_oracles::band::HandleMsg::UpdateSymbolPrice {
            base_symbol: pair.0.to_string(),
            quote_symbol: pair.1.to_string(),
            rate,
            last_updated,
        };
        self.wrap_handle(&msg, account_key)
    }
}

impl ProxyBandOracleContract {
    pub fn new(
        admin_auth: Contract,
        quote_symbol: &str,
        band: Contract,
        account_key: Option<&str>,
        name: Option<&str>,
    ) -> Result<Self> {
        let msg = proxy_band_oracle::InitMsg {
            quote_symbol: quote_symbol.to_string(),
            band,
            admin_auth,
        };
        let info = Self::wrap_init(&msg, account_key, name)?;
        Ok(ProxyBandOracleContract { info })
    }
}

impl SiennaswapSpotLpOracleContract {
    pub fn new(
        msg: &lp_oracle::siennaswap::InitMsg,
        account_key: Option<&str>,
        name: Option<&str>,
    ) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name)?;
        Ok(SiennaswapSpotLpOracleContract { info })
    }
}

impl ShadeStakingDerivativeOracleContract {
    pub fn new(
        msg: &shade_oracles::staking_derivative::shade::InitMsg,
        account_key: Option<&str>,
        name: Option<&str>,
    ) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name)?;
        Ok(ShadeStakingDerivativeOracleContract { info })
    }
}

impl EarnV1OracleContract {
    pub fn new(
        owner: String,
        deposit_token_oracle: Contract,
        strategy: Contract,
        account_key: Option<&str>,
        name: Option<&str>,
    ) -> Result<Self> {
        let msg = earn_v1_oracle::InitMsg {
            owner,
            deposit_token_oracle,
            strategy,
        };
        let info = Self::wrap_init(&msg, account_key, name)?;
        Ok(EarnV1OracleContract { info })
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
        msg: &index_oracle::InitMsg,
        account_key: Option<&str>,
        name: Option<&str>,
    ) -> Result<Self> {
        let info = Self::wrap_init(msg, account_key, name)?;
        Ok(IndexOracleContract { info })
    }

    pub fn query_config(&self, key: String) -> Result<index_oracle::Config> {
        query_contract(self.get_info(), index_oracle::QueryMsg::GetPrice { key })
    }

    pub fn query_basket(&self) -> Result<index_oracle::QueryAnswer> {
        query_contract(self.get_info(), index_oracle::QueryMsg::Basket { })
    }

    pub fn update_config(&self, router: Option<Contract>, enabled: Option<bool>, sender_key: Option<&str>) -> Result<GasLog> {
        let msg = index_oracle::HandleMsg::UpdateConfig { router, enabled };
        self.wrap_handle(&msg, sender_key)
    }

    pub fn mod_basket(&self, basket: Vec<(String, Uint128)>, sender_key: Option<&str>) -> Result<GasLog> {
        let msg = index_oracle::HandleMsg::ModBasket { basket };
        self.wrap_handle(&msg, sender_key)
    }
}
