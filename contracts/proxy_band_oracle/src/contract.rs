use cosmwasm_std::{Uint128, QueryResponse, entry_point};
use cosmwasm_std::{
    to_binary, DepsMut, Binary, Env, Deps, Response, MessageInfo, 
   StdError, StdResult,
};
use shade_oracles::common::{Oracle, oracle_exec, oracle_query};
use shade_oracles::common::querier::verify_admin;
use shade_oracles::interfaces::band::proxy::QuoteSymbol;
use shade_oracles::validate_admin;
use shade_oracles::{
    pad_handle_result, pad_query_result, Contract, ResponseStatus, BLOCK_SIZE,
    interfaces::band::{
        proxy::{Config, ExecuteMsg, InstantiateMsg},
        reference_data, reference_data_bulk, ReferenceData,
    },
    common::{is_disabled, HandleAnswer, OraclePrice, OracleQuery},
    storage::{ItemStorage, Item},
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    ProxyBandOracle.init_config(deps, msg.config)?;
    let symbol = QuoteSymbol(msg.quote_symbol);
    symbol.save(deps.storage)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    oracle_exec(deps, env, info, msg, ProxyBandOracle)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: OracleQuery) -> StdResult<QueryResponse> {
    oracle_query(deps, env, msg, ProxyBandOracle)
}

pub struct ProxyBandOracle;

impl Oracle for ProxyBandOracle {
    fn _try_query_price(&self, deps: Deps, env: &Env, key: String, config: &shade_oracles::common::CommonConfig) -> StdResult<OraclePrice> {
        if key == "SHD" {
            return to_binary(&OraclePrice::new(
                key,
                ReferenceData {
                    rate: Uint128::from(13450000000000000000u128),
                    last_updated_base: 1654019032,
                    last_updated_quote: 1654019032,
                },
            ));
        }
    
        let band_response = reference_data(
            &deps.querier,
            key.clone(),
            config.quote_symbol.clone(),
            config.band,
        )?;
    }
    fn _try_query_prices(&self, deps: Deps, env: Env, keys: Vec<String>, config: shade_oracles::common::CommonConfig) -> StdResult<Vec<OraclePrice>> {
        let quote_symbol = QuoteSymbol::load(deps.storage)?;
        let quote_symbols = vec![quote_symbol.0; keys.len()];

        let band_response =
            reference_data_bulk(&deps.querier, keys.clone(), quote_symbols, config.band)?;
    
        let mut prices: Vec<OraclePrice> = vec![];
        for (index, key) in keys.iter().enumerate() {
            prices.push(OraclePrice::new(
                key.to_string(),
                band_response[index].clone(),
            ));
        }
        Ok(prices)
    }
}