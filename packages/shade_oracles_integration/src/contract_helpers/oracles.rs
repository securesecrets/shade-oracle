use crate::constants::*;
use secretcli::{cli_types::NetContract, secretcli::query_contract};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use shade_oracles::{
    band, band::proxy as proxy_band_oracle, common as common_oracles, common::Contract,
    earn as earn_v1_oracle, lp as lp_oracle, router,
};
use cosmwasm_std::{Uint128, HumanAddr};

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

    pub fn query_config(&self, key: String) -> Result<router::ConfigResponse> {
        query_contract(self.get_info(), router::QueryMsg::GetOracle { key })
    }

    pub fn query_price(&self, key: String) -> Result<common_oracles::OraclePrice> {
        query_contract(self.get_info(), router::QueryMsg::GetPrice { key })
    }

    pub fn query_oracle(&self, key: String) -> Result<router::OracleResponse> {
        query_contract(self.get_info(), router::QueryMsg::GetOracle { key })
    }

    pub fn update_config(&self, owner: Option<HumanAddr>, default_oracle: Option<Contract>, sender_key: Option<&str>) -> Result<GasLog> {
        let msg = router::HandleMsg::UpdateConfig { owner, default_oracle };
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
    fn query_price(&self, symbol: String) -> Result<common_oracles::OraclePrice> {
        query_contract(
            self.get_info(),
            shade_oracles::common::QueryMsg::GetPrice { symbol },
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
        owner: String,
        quote_symbol: &str,
        band: Contract,
        account_key: Option<&str>,
        name: Option<&str>,
    ) -> Result<Self> {
        let msg = proxy_band_oracle::InitMsg {
            owner: HumanAddr::from(owner),
            quote_symbol: quote_symbol.to_string(),
            band,
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
