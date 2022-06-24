use cosmwasm_std::{from_binary, Binary, Env, HandleResponse, InitResponse, StdResult};
use fadroma::ensemble::{ContractHarness, MockDeps, MockEnv};

#[macro_export]
macro_rules! implement_harness {
    ($x:ident, $s:ident) => {
        impl ContractHarness for $x {
            fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
                $s::contract::init(deps, env, from_binary(&msg)?)
            }

            fn handle(
                &self,
                deps: &mut MockDeps,
                env: Env,
                msg: Binary,
            ) -> StdResult<HandleResponse> {
                $s::contract::handle(deps, env, from_binary(&msg)?)
            }

            fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
                $s::contract::query(deps, from_binary(&msg)?)
            }
        }
    };
}

use siennaswap_lp_spot_oracle;
pub struct SiennaSwapLpOracle;
implement_harness!(SiennaSwapLpOracle, siennaswap_lp_spot_oracle);

use siennaswap_lp_oracle;
pub struct SiennaSwapLp;
implement_harness!(SiennaSwapLp, siennaswap_lp_oracle);

use siennaswap_market_oracle;
pub struct SiennaMarketOracle;
implement_harness!(SiennaMarketOracle, siennaswap_market_oracle);

use shadeswap_market_oracle;
pub struct ShadeMarketOracle;
implement_harness!(ShadeMarketOracle, shadeswap_market_oracle);

use mock_band;
pub struct MockBand;
implement_harness!(MockBand, mock_band);

use mock_sienna_pair;
pub struct MockSiennaPair;
implement_harness!(MockSiennaPair, mock_sienna_pair);

use proxy_band_oracle;
pub struct ProxyBandOracle;
implement_harness!(ProxyBandOracle, proxy_band_oracle);

use index_oracle;
pub struct IndexOracle;
implement_harness!(IndexOracle, index_oracle);

use oracle_router;
pub struct OracleRouter;
implement_harness!(OracleRouter, oracle_router);

use snip20_reference_impl;
pub struct Snip20;
implement_harness!(Snip20, snip20_reference_impl);

use mock_shade_pair;
pub struct MockShadePair;
implement_harness!(MockShadePair, mock_shade_pair);

use admin;
pub struct AdminAuth;
implement_harness!(AdminAuth, admin);