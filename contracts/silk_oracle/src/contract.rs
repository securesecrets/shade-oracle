use mulberry_utils::{
    auth::{assert_admin, load_admin, save_admin},
    common::types::{CanonicalContract, Contract, ResponseStatus},
    scrt::{
        to_binary, Api, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
        QueryResult, StdError, StdResult, Storage, BLOCK_SIZE,
    },
    secret_toolkit::utils::{pad_handle_result, pad_query_result, Query},
    storage::bincode_state::{load, save},
};
use serde::{Deserialize, Serialize};
use shade_oracles::{
    band::{
        proxy::{ConfigResponse, HandleAnswer},
        BandQuery,
    },
    common::{PriceResponse, QueryMsg},
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static BASKET: &[u8] = b"basket";

#[derive(Serialize, Deserialize)]
pub struct InitMsg {
    band: Contract,
    basket: Vec<Asset>,
}

#[derive(Serialize, Deserialize)]
pub enum HandleMsg {
    // Asset with weight 0 will be removed
    // all others are added or changed
    ModBasket {
        basket: Vec<Asset>,
    },
    UpdateConfig {
        owner: HumanAddr,
        band: Contract,
    },
}

/*
#[derive(Serialize, Deserialize)]
pub enum HandleAnswer { }
*/

/* TODO
 * Router could accept a list of symbols, return a list of routes
 * This would allow the end oracle to combine queries to the same oracle
 */

#[derive(Serialize, Deserialize)]
pub struct State {
    pub band: Contract,
    pub basket: Vec<Asset>,
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state: State = State {
        band: msg.band,
    };

    save_admin(deps, &msg.owner)?;
    save(&mut deps.storage, CONFIG_KEY, &state)?;
    save(&mut deps.storage, BASKET, &msg.basket)?;

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
        } => try_update_config(deps, env, owner, band),
        HandleMsg::ModBasket{ basket, .. } => mod_basket(deps, env, basket),
    };
    pad_handle_result(response, BLOCK_SIZE)
}

fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<String>,
    band: Option<Contract>,
) -> StdResult<HandleResponse> {
    let mut state: State = load(&deps.storage, CONFIG_KEY)?;

    assert_admin(deps, &env)?;

    if let Some(owner) = owner {
        save_admin(deps, &HumanAddr(owner))?;
    }

    if let Some(band) = band {
        state.band = band;
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

fn mod_basket<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    basket: Vec<Asset>,
) -> StdResult<HandleResponse> {

    assert_admin(deps, &env)?;

    let mut cur_basket: Vec<Asset> = load(&deps.storage, BASKET)?;
    let cur_symbols = cur_basket.iter().map(|a| a.symbol).collect();

    // TODO Disallow adding the symbol for this asset

    for mod_asset in basket {
        match cur_basket.iter().position(|a| a.symbol == mod_asset.symbol) {
            Some(i) => {
                if mod_asset.weight > Uint128::zero() {
                    // Update
                    cur_basket[i].weight = mod_asset.weight;
                }
                else {
                    // Remove
                    cur_basket.remove(i);
                }
            },
            None => {
                // Add new
                cur_basket.push(mod_asset);
            }
        }
    }

    cur_basket.sort_by(|a, b| a.symbol.cmp(&b.symbol));

    save(&mut deps.storage, BASKET, &cur_basket)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn eval_index<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    basket: Vec<Asset>,
) -> StdResult<cosmwasm_std::Uint128> {

    let config: State = load(&deps.storage, CONFIG_KEY)?;

    let mut weight_sum = Uint512::zero();
    let mut price = Uint512::zero();

    let mut band_bases = vec![];
    let mut band_quotes = vec![];
    let mut band_weights = vec![];

    for asset in index {
        weight_sum += Uint512::from(asset.weight.u128());

        // query oracle

        if false { //TODO Check source
            // Non-band assets
            return Err(StdError::generic_err(format!(
                "EVAL INDEX DEX PAIRS {}",
                asset.symbol
            )));

        }
        else {
            // Combine band assets for bulk query
            band_weights.push(element.weight);
            band_bases.push(element.symbol.clone());
            band_quotes.push("USD".to_string());
        }
    }

    if band_bases.len() > 0 {
        //TODO get from shade implementation
        let ref_data = band::reference_data_bulk(
            deps,
            band_bases,
            band_quotes,
            config.band,
        )?;

        for (reference, weight) in ref_data.iter().zip(band_weights.iter()) {
            price += Uint512::from(reference.rate.u128()) * Uint512::from(weight.u128())
                / Uint512::from(10u128.pow(18));
        }
    }

    Ok(cosmwasm_std::Uint128(
        Uint128::try_from(
            price
                .checked_mul(Uint512::from(10u128.pow(18)))?
                .checked_div(weight_sum)?,
        )?
        .u128(),
    ))
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    let response = match msg {
        QueryMsg::GetConfig {} => to_binary(&try_query_config(deps)?),
        /* add 'symbol' so we can error if its the wrong oracle
         * Prevents router failure from causing economic failure
         */
        QueryMsg::GetPrice { .. } => to_binary(&try_query_price(deps)?),
        //QueryMsg::GetPrices { symbols: Vec<String> } => to_binary(&try_query_price(deps)?),
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

    let price = eval_index(&deps, load(&deps.storage, BASKET)?)?;

    BandQuery::GetReferenceData {
        base_symbol: state.base_symbol,
        quote_symbol: state.quote_symbol,
    }
    .query(
        &deps.querier,
        state.band.code_hash,
        state.band.address,
    )
}
