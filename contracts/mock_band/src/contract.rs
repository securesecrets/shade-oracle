use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_oracles::band::{HandleAnswer, HandleMsg, InitMsg, ReferenceData};
use shade_oracles::scrt::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdError, StdResult, Storage, Uint128,
};
use shade_oracles::storage::Map;
use shade_oracles::{
    common::ResponseStatus,
};

#[derive(Serialize, Deserialize, Default, JsonSchema)]
pub struct SavedBandData {
    pub rate: u128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

const MOCK_DATA: Map<(&str, &str), SavedBandData> = Map::new("price-data");

pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateSymbolPrice {
            base_symbol,
            quote_symbol,
            rate,
            last_updated,
        } => update_symbol_price(
            deps,
            env,
            base_symbol,
            quote_symbol,
            rate,
            last_updated,
        ),
    }
}

pub fn update_symbol_price<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    base_symbol: String,
    quote_symbol: String,
    rate: Uint128,
    last_updated: Option<u64>,
) -> StdResult<HandleResponse> {

    MOCK_DATA.update(&mut deps.storage, (base_symbol.as_str(), quote_symbol.as_str()), |_data| -> StdResult<_> {
        let mut new_data: SavedBandData = SavedBandData {
            rate: rate.u128(),
            last_updated_base: env.block.time,
            last_updated_quote: env.block.time,
        };
    
        if let Some(last_updated) = last_updated {
            new_data.last_updated_base = last_updated;
            new_data.last_updated_quote = last_updated;
        }
        Ok(new_data)
    })?;


    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateSymbolPrice { status: ResponseStatus::Success })?),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
    GetReferenceDataBulk {
        base_symbols: Vec<String>,
        quote_symbols: Vec<String>,
    },
}
pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetReferenceData {
            base_symbol,
            quote_symbol,
        } => query_saved_band_data(deps, base_symbol, quote_symbol),
        QueryMsg::GetReferenceDataBulk {
            base_symbols,
            quote_symbols: _,
        } => {
            let mut results = Vec::new();
            let data = ReferenceData {
                rate: Uint128(1_000_000_000_000_000_000),
                last_updated_base: 1628544285u64,
                last_updated_quote: 3377610u64,
            };

            for _ in base_symbols {
                results.push(data.clone());
            }
            to_binary(&results)
        }
    }
}

fn query_saved_band_data<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    base_symbol: String,
    quote_symbol: String,
) -> StdResult<Binary> {

    let saved_band_data =
        MOCK_DATA.may_load(&deps.storage, (base_symbol.as_str(), quote_symbol.as_str()));

    match saved_band_data {
        Ok(saved_band_data) => {
            if let Some(saved_band_data) = saved_band_data {
                to_binary(&ReferenceData {
                    rate: Uint128(saved_band_data.rate),
                    last_updated_base: saved_band_data.last_updated_base,
                    last_updated_quote: saved_band_data.last_updated_quote,
                })
            } else {
                to_binary(&ReferenceData {
                    rate: Uint128(1_000_000_000_000_000_000),
                    last_updated_base: 1628544285u64,
                    last_updated_quote: 3377610u64,
                })
            }
        }
        Err(_) => to_binary(&StdError::GenericErr {
            msg: "Failed to load from storage.".to_string(),
            backtrace: None,
        }),
    }
}
