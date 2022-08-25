use super::*;
use crate::{
    interfaces::band::{reference_data, reference_data_bulk},
    interfaces::{
        router::msg::{
            ConfigResponse as RouterConfigResponse, OracleResponse, QueryMsg as RouterQueryMsg,
        },
    },
};
use cosmwasm_std::{QuerierWrapper, StdResult};
use shade_protocol::{
    contract_interfaces::snip20::{QueryAnswer as Snip20QueryAnswer, QueryMsg as Snip20QueryMsg},
    snip20::helpers::{token_info, TokenInfo},
    Contract,
};
use std::collections::HashMap;

use super::OraclePrice;

pub fn query_oracle_price(
    oracle: &Contract,
    querier: &QuerierWrapper,
    key: impl Into<String>,
) -> StdResult<OraclePrice> {
    let resp: PriceResponse = OracleQuery::GetPrice { key: key.into() }.query(querier, oracle)?;
    Ok(resp.price)
}

/// Gets the oracle for the key from the router & calls GetPrice on it.
///
/// Has a query depth of 1.
pub fn query_price(
    router: &Contract,
    querier: &QuerierWrapper,
    key: impl Into<String>,
) -> StdResult<OraclePrice> {
    let key: String = key.into();
    let oracle_resp: OracleResponse =
        RouterQueryMsg::GetOracle { key: key.clone() }.query(querier, router)?;
    query_oracle_price(&oracle_resp.oracle, querier, key)
}

pub fn query_oracle_prices(
    oracle: &Contract,
    querier: &QuerierWrapper,
    keys: Vec<String>,
) -> StdResult<Vec<OraclePrice>> {
    let resp: PricesResponse = OracleQuery::GetPrices { keys }.query(querier, oracle)?;
    Ok(resp.prices)
}

/// Groups the keys by their respective oracles and sends bulk GetPrices queries to each of those oracles.
///
/// Done to reduce impact on query depth.
pub fn query_prices<'a>(
    router: &Contract,
    querier: &QuerierWrapper,
    keys: impl IntoIterator<Item = &'a String>,
) -> StdResult<Vec<OraclePrice>> {
    let oracle_resps: Vec<OracleResponse> = RouterQueryMsg::GetOracles {
        keys: keys.into_iter().map(|s| s.to_string()).collect(),
    }
    .query(querier, router)?;
    let mut map: HashMap<Contract, Vec<String>> = HashMap::new();
    let mut prices: Vec<OraclePrice> = vec![];

    for resp in oracle_resps {
        // Get the current vector of symbols at that oracle and add the current key to it
        map.entry(resp.oracle).or_insert(vec![]).push(resp.key);
    }

    for (oracle, keys) in map {
        if keys.len() == 1 {
            let queried_price = query_oracle_price(&oracle, querier, keys[0].clone())?;
            prices.push(queried_price);
        } else {
            let mut queried_prices = query_oracle_prices(&oracle, querier, keys)?;
            prices.append(&mut queried_prices);
        }
    }
    Ok(prices)
}

pub fn query_band_price(
    router: &Contract,
    querier: &QuerierWrapper,
    key: impl Into<String>,
) -> StdResult<OraclePrice> {
    let resp: RouterConfigResponse = RouterQueryMsg::GetConfig {}.query(querier, router)?;
    let config = resp.config;
    let key: String = key.into();
    let band_response = reference_data(
        querier,
        key.clone(),
        config.quote_symbol.clone(),
        &config.band,
    )?;
    Ok(OraclePrice::new(key, band_response))
}

pub fn query_band_prices<'a>(
    router: &Contract,
    querier: &QuerierWrapper,
    keys: impl IntoIterator<Item = &'a String>,
) -> StdResult<Vec<OraclePrice>> {
    let resp: RouterConfigResponse = RouterQueryMsg::GetConfig {}.query(querier, router)?;
    let config = resp.config;
    let mut prices: Vec<OraclePrice> = vec![];
    let base_symbols = keys
        .into_iter()
        .map(|key| {
            prices.push(OraclePrice::new(key.to_string(), ReferenceData::default()));
            key.to_string()
        })
        .collect::<Vec<String>>();
    let prices_count = base_symbols.len();
    let quote_symbols = vec![config.quote_symbol; prices_count];

    let band_data = reference_data_bulk(querier, base_symbols, quote_symbols, &config.band)?;

    for i in 0..prices_count {
        prices[i].data = band_data[i].clone();
    }

    Ok(prices)
}

/// Gets the admin auth contract from the router and uses it to check if the user is an admin for the router.
pub fn verify_admin(
    contract: &Contract,
    querier: &QuerierWrapper,
    user: impl Into<String> + Clone,
) -> StdResult<()> {
    let get_admin_auth_req: RouterConfigResponse =
        RouterQueryMsg::GetConfig {}.query(querier, contract)?;
    let admin_auth = get_admin_auth_req.config.admin_auth;
    validate_permission(
        querier,
        ShadeOraclePermissions::SuperAdmin,
        &user,
        &admin_auth,
    )
}

pub fn validate_permission(
    querier: &QuerierWrapper,
    permission: ShadeOraclePermissions,
    user: &(impl Into<String> + Clone),
    admin_auth: &(impl Into<Contract> + Clone),
) -> StdResult<()> {
    shade_admin::querier::validate_permission(querier, &permission.to_string(), user, admin_auth)
}

pub fn query_token_info(contract: &Contract, querier: &QuerierWrapper) -> StdResult<TokenInfo> {
    token_info(querier, contract)
}

pub fn query_token_balance(
    contract: &Contract,
    querier: &QuerierWrapper,
    address: impl Into<String>,
    key: impl Into<String>,
) -> StdResult<Uint128> {
    let answer: Snip20QueryAnswer = Snip20QueryMsg::Balance {
        address: address.into(),
        key: key.into(),
    }
    .query(querier, contract)?;
    match answer {
        Snip20QueryAnswer::Balance { amount } => Ok(amount),
        Snip20QueryAnswer::ViewingKeyError { msg } => Err(StdError::generic_err(msg)),
        _ => Err(StdError::generic_err(
            "Invalid response to query token balance.",
        )),
    }
}
