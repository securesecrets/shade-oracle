use cosmwasm_std::{entry_point, Deps, Env, Response, StdError, StdResult};
use cosmwasm_std::{DepsMut, MessageInfo, QueryResponse, Uint128};
use shade_oracles::core::Query;
use shade_oracles::interfaces::dex::lp_market::InstantiateMsg;
use shade_oracles::{
    common::querier::{query_band_price, query_price, query_token_info},
    core::Contract,
    interfaces::band::ReferenceData,
    interfaces::dex::shadeswap::market::*,
    protocols::shadeswap::*,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    oracle_exec(deps, _env, info, msg, ShadeswapMarketOracle)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: OracleQuery) -> StdResult<QueryResponse> {
    oracle_query(deps, _env, msg, ShadeswapMarketOracle)
}

pub struct ShadeswapMarketOracle;

impl Oracle for ShadeswapMarketOracle {
    fn try_query_price(
        &self,
        deps: Deps,
        _env: &Env,
        key: String,
        config: &shade_oracles::common::CommonConfig,
    ) -> StdResult<OraclePrice> {
        let primary_token = PRIMARY_TOKEN.load(deps.storage)?;
        let market_config = MarketData::load(deps.storage)?;
        let primary_info = PRIMARY_INFO.load(deps.storage)?;

        // Simulate trade 1 primary -> 1 base
        let sim: EstimatedPriceResponse = ShadeSwapQueryMsg::SwapSimulation {
            offer: TokenAmount {
                amount: Uint128::from(10u128.pow(primary_info.decimals.into())),
                token: TokenType::CustomToken {
                    contract_addr: primary_token.address,
                    token_code_hash: primary_token.code_hash,
                    oracle_key: _,
                },
            },
            exclude_fee: Some(true),
        }
        .query(&deps.querier, &market_config.pair)?;

        // Normalize to 'rate * 10^18'
        let base_info = BASE_INFO.load(deps.storage)?;
        let exchange_rate = normalize_price_uint128(sim.estimated_price, base_info.decimals)?;

        // Query router for base_peg/USD
        let base_usd_price = if config.only_band {
            query_band_price(&config.router, &deps.querier, market_config.base_peg)?
        } else {
            query_price(&config.router, &deps.querier, market_config.base_peg)?
        };

        // Translate price to primary/USD
        let price = base_usd_price
            .data()
            .rate
            .multiply_ratio(exchange_rate, 10u128.pow(18));

        Ok(OraclePrice::new(
            key,
            ReferenceData {
                rate: price,
                last_updated_base: base_usd_price.data().last_updated_base,
                last_updated_quote: base_usd_price.data().last_updated_quote,
            },
        ))
    }
}
