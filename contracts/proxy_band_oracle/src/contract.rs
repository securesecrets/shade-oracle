use cosmwasm_std::{Uint128, QueryResponse, entry_point};
use cosmwasm_std::{
    DepsMut, Env, Deps, Response, MessageInfo, StdResult,
};
use shade_oracles::core::Contract;
use shade_oracles::common::{Oracle, oracle_exec, oracle_query, ExecuteMsg};
use shade_oracles::interfaces::band::proxy::QuoteSymbol;
use shade_oracles::{
    interfaces::band::{
        proxy::{InstantiateMsg},
        reference_data, reference_data_bulk, ReferenceData,
    },
    common::{OraclePrice, OracleQuery},
    storage::{ItemStorage, Item},
};

const BAND: Item<Contract> = Item::new("band-contract");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let symbol = QuoteSymbol(msg.quote_symbol);
    symbol.save(deps.storage)?;
    BAND.save(deps.storage, &msg.band.into_valid(deps.api)?)?;
    ProxyBandOracle.init_config(deps.storage, deps.api, msg.config)?;

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
    fn try_query_price(&self, deps: Deps, _env: &Env, key: String, _config: &shade_oracles::common::CommonConfig) -> StdResult<OraclePrice> {
        let band = BAND.load(deps.storage)?;
        if key == "SHD" {
            return Ok(OraclePrice::new(
                key,
                ReferenceData {
                    rate: Uint128::from(13450000000000000000u128),
                    last_updated_base: 1654019032,
                    last_updated_quote: 1654019032,
                },
            ));
        }
        let quote_symbol = QuoteSymbol::load(deps.storage)?;
        let band_response = reference_data(
            &deps.querier,
            key.clone(),
            quote_symbol.0,
            &band,
        )?;
        Ok(OraclePrice::new(key, band_response))
    }
    fn try_query_prices(&self, deps: Deps, _env: &Env, keys: Vec<String>, _config: &shade_oracles::common::CommonConfig) -> StdResult<Vec<OraclePrice>> {
        let quote_symbol = QuoteSymbol::load(deps.storage)?;
        let quote_symbols = vec![quote_symbol.0; keys.len()];
        let band = BAND.load(deps.storage)?;

        let band_response =
            reference_data_bulk(&deps.querier, keys.clone(), quote_symbols, &band)?;
    
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