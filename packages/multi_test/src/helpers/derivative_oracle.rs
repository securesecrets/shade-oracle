use super::{
    AnyResult, App, AppResponse, Contract, OracleRouter, PriceResponse, PricesResponse,
    RawContract, StdResult, User,
};
use crate::harness::derivative_oracle::DerivativeOracle;
use crate::helpers::TestScenario;
use derivative_oracle::msg::*;
use shade_oracles::interfaces::router;
use shade_oracles::{core::Query, interfaces::providers::RawProvider, status::ContractStatus};
use shade_toolkit::{
    multi::{MultiTestable, Tester},
    ExecuteCallback, InstantiateCallback,
};

create_test_helper!(DerivativeOracleHelper);

impl DerivativeOracleHelper {
    pub fn init(
        user: &User,
        app: &mut App,
        router: &Contract,
        derivative: &Contract,
        admin_auth: &Contract,
        underlying_key: String,
        price_key: String,
        rate_key: String,
    ) -> Self {
        let contract = user
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                    derivative: derivative.clone().into(),
                    admin_auth: admin_auth.clone().into(),
                    underlying_key,
                    price_key,
                    rate_key,
                },
                DerivativeOracle::default(),
                "derivative_oracle",
            )
            .unwrap();
        Self(contract)
    }

    pub fn update_config(
        &self,
        sender: &User,
        app: &mut App,
        router: Option<RawContract>,
        derivative: Option<RawContract>,
        admin_auth: Option<RawContract>,
        underlying_key: Option<String>,
        price_key: Option<String>,
        rate_key: Option<String>,
        enabled: Option<bool>,
    ) -> AnyResult<AppResponse> {
        sender.exec(
            app,
            &ExecuteMsg::UpdateConfig {
                router,
                derivative,
                admin_auth,
                underlying_key,
                price_key,
                rate_key,
                enabled,
            },
            &self.0,
        )
    }

    pub fn query_config(&self, app: &App) -> StdResult<Config> {
        QueryMsg::GetConfig {}.test_query(&self.0, app)
    }

    pub fn query_price(&self, app: &App, key: String) -> StdResult<PriceResponse> {
        QueryMsg::GetPrice { key }.test_query(&self.0, app)
    }

    pub fn query_prices(&self, app: &App, keys: Vec<String>) -> StdResult<PricesResponse> {
        QueryMsg::GetPrices { keys }.test_query(&self.0, app)
    }
}
