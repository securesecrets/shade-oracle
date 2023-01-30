use super::*;
use crate::harness::derivatives::ShadeStakingDerivativesOracle;
use shade_oracles::interfaces::{common::config::CommonConfigResponse, derivatives::generic::*};

create_test_helper!(GenericStakingDerivativesOracleHelper);

impl GenericStakingDerivativesOracleHelper {
    pub fn init_shade_v1(sender: &User, app: &mut App, router: &Contract) -> Self {
        let contract = sender
            .init(
                app,
                &InstantiateMsg {
                    router: router.clone().into(),
                },
                ShadeStakingDerivativesOracle::default(),
                "shade_staking_derivatives_oracle",
            )
            .unwrap();
        Self(contract)
    }

    pub fn set_derivatives(
        &self,
        sender: &User,
        app: &mut App,
        derivatives: &[RawDerivativeData],
    ) -> AnyResult<AppResponse> {
        sender.exec(
            app,
            &ExecuteMsg::SetDerivatives(derivatives.to_vec()),
            &self.0,
        )
    }

    pub fn remove_derivatives(
        &self,
        sender: &User,
        app: &mut App,
        keys: &[String],
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::RemoveDerivatives(keys.to_vec()), &self.0)
    }

    pub fn update_assets(
        &self,
        sender: &User,
        app: &mut App,
        assets: &[RawAsset],
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::UpdateAssets(assets.to_vec()), &self.0)
    }

    pub fn update_config(
        &self,
        sender: &User,
        app: &mut App,
        router: &RawContract,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::UpdateConfig(router.clone()), &self.0)
    }

    pub fn set_status(&self, sender: &User, app: &mut App, status: bool) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::SetStatus(status), &self.0)
    }

    pub fn query_derivatives(&self, app: &App) -> StdResult<DerivativesResponse> {
        QueryMsg::GetDerivatives {}.test_query(&self.0, app)
    }
    pub fn query_config(&self, app: &App) -> StdResult<CommonConfigResponse> {
        QueryMsg::GetConfig {}.test_query(&self.0, app)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use shade_oracles::{
        interfaces::{band::ReferenceData, common::OraclePrice},
        unit_test_interface::prices::PricesFixture,
    };

    #[test]
    fn test_stride_registry() {
        let prices = PricesFixture::basic_prices_2();
        let TestScenario {
            mut app,
            admin,
            user,
            router,
            ..
        } = TestScenario::new(prices);
        let app = &mut app;
        let oracle =
            GenericStakingDerivativesOracleHelper::init_shade_v1(&user, app, &router.into());
        let tokens = Snip20Helper::generate_tokens(
            app,
            &user,
            vec![("stkdBTC", "SBTC", 10), ("stkdETH", "SETH", 10)],
        );
        assert!(oracle.set_status(&user, app, false).is_err());
        assert!(oracle.set_status(&admin, app, false).is_ok());

        let stkd_eth_asset = tokens[1].to_raw_asset("ETH");
        let stkd_btc_asset = tokens[0].to_raw_asset("BTC");
        let derivatives = vec![RawDerivativeData {
            staking_derivative: stkd_eth_asset,
            key: "stkd-ETH".to_string(),
        }];
        assert!(oracle.set_derivatives(&admin, app, &derivatives).is_err());
    }
}
