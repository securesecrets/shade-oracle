use mulberry_utils::{
    auth::{assert_admin, load_admin, save_admin},
    common::types::{CanonicalContract, Contract, ResponseStatus},
    scrt::{
        debug_print, to_binary, Api, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
        QueryResult, StdError, StdResult, Storage, BLOCK_SIZE,
    },
    secret_toolkit::utils::{pad_handle_result, pad_query_result, Query},
    storage::bincode_state::{load, save},
};
use serde::{Deserialize, Serialize};
use shade_oracles::{
    band::{
        proxy::{ConfigResponse, HandleAnswer, HandleMsg, InitMsg},
        BandQuery,
    },
    common::{PriceResponse, QueryMsg},
};

pub static CONFIG_KEY: &[u8] = b"config";

/// state of the auction
#[derive(Serialize, Deserialize)]
pub struct State {
    pub band: CanonicalContract,
    pub base_symbol: String,
    pub quote_symbol: String,
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let band_contract: CanonicalContract = CanonicalContract {
        address: deps.api.canonical_address(&HumanAddr(msg.band.address))?,
        code_hash: msg.band.code_hash,
    };

    let state: State = State {
        band: band_contract,
        base_symbol: msg.base_symbol,
        quote_symbol: msg.quote_symbol,
    };

    save_admin(deps, &HumanAddr(msg.owner))?;
    save(&mut deps.storage, CONFIG_KEY, &state)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let response: Result<HandleResponse, StdError> = match msg {
        HandleMsg::UpdateConfig {
            owner,
            band,
            base_symbol,
            quote_symbol,
        } => try_update_config(deps, env, owner, band, base_symbol, quote_symbol),
    };
    pad_handle_result(response, BLOCK_SIZE)
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<String>,
    band: Option<Contract>,
    base_symbol: Option<String>,
    quote_symbol: Option<String>,
) -> StdResult<HandleResponse> {
    let mut state: State = load(&deps.storage, CONFIG_KEY)?;

    assert_admin(deps, &env)?;

    if let Some(owner) = owner {
        save_admin(deps, &HumanAddr(owner))?;
    }

    if let Some(band) = band {
        let canonical_band = CanonicalContract {
            address: deps.api.canonical_address(&HumanAddr(band.address))?,
            code_hash: band.code_hash,
        };
        state.band = canonical_band;
    }

    if let Some(base_symbol) = base_symbol {
        state.base_symbol = base_symbol;
    }

    if let Some(quote_symbol) = quote_symbol {
        state.quote_symbol = quote_symbol;
    }

    save(&mut deps.storage, CONFIG_KEY, &state)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    let response = match msg {
        QueryMsg::GetConfig {} => to_binary(&try_query_config(deps)?),
        QueryMsg::GetPrice { .. } => to_binary(&try_query_price(deps)?),
    };
    pad_query_result(response, BLOCK_SIZE)
}

fn try_query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let state: State = load(&deps.storage, CONFIG_KEY)?;

    Ok(ConfigResponse {
        owner: load_admin(deps)?.to_string(),
        band: Contract {
            address: deps.api.human_address(&state.band.address)?.to_string(),
            code_hash: state.band.code_hash,
        },
        base_symbol: state.base_symbol,
        quote_symbol: state.quote_symbol,
    })
}

fn try_query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<PriceResponse> {
    let state: State = load(&deps.storage, CONFIG_KEY)?;

    BandQuery::GetReferenceData {
        base_symbol: state.base_symbol,
        quote_symbol: state.quote_symbol,
    }
    .query(
        &deps.querier,
        state.band.code_hash,
        deps.api.human_address(&state.band.address)?,
    )
}
