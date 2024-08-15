use super::{
    AnyResult, App, AppResponse, Contract, OracleRouter, PriceResponse, PricesResponse,
    RawContract, StdResult, User,
};
use crate::harness::dshd_oracle::DShdOracle;
use crate::helpers::TestScenario;
use dshd_oracle::msg::*;
use shade_oracles::interfaces::router;
use shade_oracles::{core::Query, interfaces::providers::RawProvider, status::ContractStatus};
use shade_toolkit::{
    multi::{MultiTestable, Tester},
    ExecuteCallback, InstantiateCallback,
};

create_test_helper!(DShdOracleHelper);

impl DShdOracleHelper {
    pub fn init(
        user: &User,
        app: &mut App,
        router: &Contract,
        dshd: &Contract,
        admin_auth: &Contract,
    ) -> Self {
        let contract = user
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                    dshd: dshd.clone().into(),
                    admin_auth: admin_auth.clone().into(),
                },
                DShdOracle::default(),
                "dshd_oracle",
            )
            .unwrap();
        Self(contract)
    }

    pub fn update_config(
        &self,
        sender: &User,
        app: &mut App,
        router: Option<RawContract>,
        dshd: Option<RawContract>,
        admin_auth: Option<RawContract>,
        enabled: Option<bool>,
    ) -> AnyResult<AppResponse> {
        sender.exec(
            app,
            &ExecuteMsg::UpdateConfig {
                router,
                dshd,
                admin_auth,
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
