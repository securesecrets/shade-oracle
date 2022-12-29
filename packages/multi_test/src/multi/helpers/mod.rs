use crate::multi::{MockBand, OracleRouter};
use cosmwasm_std::{Addr, ContractInfo, StdResult, Uint128, Uint256};
use shade_multi_test::multi::admin::init_admin_auth;
use shade_oracles::core::Query;
use shade_oracles::interfaces::band::MockPrice;
use shade_oracles::interfaces::band::{self};
use shade_oracles::interfaces::common::{PriceResponse, PricesResponse};
use shade_oracles::interfaces::router::registry::{RegistryOperation, UpdateConfig};
use shade_protocol::multi_test::AppResponse;
use shade_protocol::{
    multi_test::App,
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable},
    AnyResult, Contract,
};
use std::collections::HashMap;

use self::router::OracleRouterHelper;

use super::*;
pub mod dex;
pub mod router;

pub struct BandHelper(pub ContractInfo);

impl BandHelper {
    pub fn update_prices(
        &self,
        sender: &Addr,
        app: &mut App,
        prices: HashMap<String, Uint128>,
        last_updated_time: Option<u64>,
    ) {
        let mut mock_prices = vec![];
        for (sym, price) in prices {
            mock_prices.push(MockPrice {
                base_symbol: sym,
                quote_symbol: "USD".into(),
                rate: price,
                last_updated: last_updated_time,
            });
        }
        band::ExecuteMsg::SetPrices(mock_prices)
            .test_exec(&self.0, app, sender.clone(), &[])
            .unwrap();
    }
}

pub struct OracleCore {
    pub band: BandHelper,
    pub router: OracleRouterHelper,
    pub admin_auth: ContractInfo,
    pub superadmin: Addr,
}

impl OracleCore {
    pub fn new(
        band: BandHelper,
        router: OracleRouterHelper,
        admin_auth: ContractInfo,
        superadmin: Addr,
    ) -> Self {
        OracleCore {
            band,
            router,
            admin_auth,
            superadmin,
        }
    }
    /// Initializes the core dependencies for testing all oracles which are
    /// band, proxy band, router, and the admin auth contract. Then, it updates the prices in band
    /// based off the prices argument with them being quoted in "USD".
    pub fn setup(
        app: &mut App,
        admin: &Addr,
        prices: HashMap<String, Uint128>,
        band: Option<ContractInfo>,
        oracle_router: Option<ContractInfo>,
        admin_auth: Option<ContractInfo>,
    ) -> AnyResult<Self> {
        let quote_symbol = "USD".to_string();
        let superadmin = admin.clone();

        let admin_auth = admin_auth.unwrap_or_else(|| init_admin_auth(app, admin));
        let mut initial_prices = vec![];
        // Configure mock band prices
        for (sym, price) in prices {
            initial_prices.push((sym, quote_symbol.clone(), price));
        }

        let band = band.unwrap_or_else(|| {
            band::InstantiateMsg { initial_prices }
                .test_init(MockBand::default(), app, admin.clone(), "band", &[])
                .unwrap()
        });

        let oracle_router = oracle_router.unwrap_or_else(|| {});

        router::msg::ExecuteMsg::UpdateConfig(UpdateConfig {
            admin_auth: None,
            band: None,
            quote_symbol: None,
        })
        .test_exec(&oracle_router, app, admin.clone(), &[])
        .unwrap();

        Ok(OracleCore::new(
            BandHelper(band),
            OracleRouterHelper(oracle_router),
            admin_auth,
            superadmin,
        ))
    }

    pub fn update_prices(
        &self,
        app: &mut App,
        prices: HashMap<String, Uint128>,
        last_updated_time: u64,
    ) {
        self.band
            .update_prices(&self.superadmin, app, prices, Some(last_updated_time));
    }

    pub fn set_keys(&self, app: &mut App, oracle: Contract, keys: Vec<String>) {
        self.router
            .set_keys(&self.superadmin, app, oracle, keys)
            .unwrap();
    }

    pub fn remove_keys(&self, app: &mut App, keys: Vec<String>) {
        self.router
            .remove_keys(&self.superadmin, app, keys)
            .unwrap();
    }
}
