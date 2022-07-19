use cosmwasm_std::{
    MessageInfo, Env, Deps, Response, DepsMut,
     StdResult, QueryResponse, entry_point,
};
use shade_oracles::{
    core::get_precision,
    interfaces::{band::ReferenceData, staking_derivative::shade::StakingDerivativeConfig},
    common::{
        querier::{query_token_info, query_band_price, query_price},
        ExecuteMsg, OraclePrice, OracleQuery, oracle_exec, Oracle, oracle_query
    },
    interfaces::staking_derivative::shade::{
        querier::query_derivative_price,
        {InstantiateMsg},
    },
    storage::{ItemStorage},
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let staking_derivative_token = msg.staking_derivative_token.into_valid(deps.api)?;
    let token_decimals = query_token_info(&staking_derivative_token, &deps.querier)?
        .decimals;

    StakingDerivativeConfig { token_decimals, staking_derivative_token, underlying_symbol: msg.underlying_symbol }.save(deps.storage)?;

    ShadeStakingDerivativeOracle.init_config(deps.storage, deps.api, msg.config)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    oracle_exec(deps, env, info, msg, ShadeStakingDerivativeOracle)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: OracleQuery) -> StdResult<QueryResponse> {
    oracle_query(deps, env, msg, ShadeStakingDerivativeOracle)
}

pub struct ShadeStakingDerivativeOracle;
impl Oracle for ShadeStakingDerivativeOracle {
    fn try_query_price(&self, deps: Deps, _env: &Env, key: String, config: &shade_oracles::common::CommonConfig) -> StdResult<OraclePrice> {
        let oracle_config = StakingDerivativeConfig::load(deps.storage)?;

        // price of underlying asset to 10^18.
        let underlying_price = if config.only_band {
            query_band_price(&config.router, &deps.querier, oracle_config.underlying_symbol)?
        } else {
            query_price(&config.router, &deps.querier, oracle_config.underlying_symbol)?
        };

        let staking_derivative_price =
            query_derivative_price(deps, &oracle_config.staking_derivative_token)?;
    
        let staking_derivative_price_precision = get_precision(oracle_config.token_decimals);
    
        let price = underlying_price
            .data
            .rate
            .multiply_ratio(staking_derivative_price, staking_derivative_price_precision);
    
        let response = ReferenceData {
            rate: price,
            last_updated_base: underlying_price.data.last_updated_base,
            last_updated_quote: underlying_price.data.last_updated_quote,
        };
        Ok(OraclePrice::new(key, response))
    }
}