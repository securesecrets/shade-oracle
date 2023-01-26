use crate::{
    interfaces::band::{reference_data, reference_data_bulk},
    interfaces::{
        band::ReferenceData,
        common::{OraclePrice, OracleQuery, PriceResponse, PricesResponse},
        router::msg::{ConfigResponse as RouterConfigResponse, QueryMsg as RouterQueryMsg},
    },
};
use cosmwasm_std::{QuerierWrapper, StdError, StdResult, Uint128};
use shade_protocol::{
    contract_interfaces::{
        admin::helpers::{validate_admin, AdminPermissions},
        snip20::{QueryAnswer as Snip20QueryAnswer, QueryMsg as Snip20QueryMsg},
    },
    snip20::helpers::{token_info, TokenInfo},
    utils::Query,
    Contract,
};

pub fn query_price(
    oracle: &Contract,
    querier: &QuerierWrapper,
    key: impl Into<String>,
) -> StdResult<PriceResponse> {
    OracleQuery::GetPrice { key: key.into() }.query(querier, oracle)
}

pub fn query_prices(
    oracle: &Contract,
    querier: &QuerierWrapper,
    keys: &[String],
) -> StdResult<PricesResponse> {
    OracleQuery::GetPrices {
        keys: keys.to_vec(),
    }
    .query(querier, oracle)
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

/// Gets the admin auth contract from the router and uses it to check if the user is an oracle admiin.
pub fn require_admin(
    contract: &Contract,
    querier: &QuerierWrapper,
    user: impl Into<String> + Clone,
) -> StdResult<()> {
    let get_admin_auth_req: RouterConfigResponse =
        RouterQueryMsg::GetConfig {}.query(querier, contract)?;
    let admin_auth = get_admin_auth_req.config.admin_auth;
    validate_admin(querier, AdminPermissions::OraclesAdmin, user, &admin_auth)
}

/// Gets the admin auth contract from the router and uses it to check if the user is an oracle bot.
pub fn require_bot(
    contract: &Contract,
    querier: &QuerierWrapper,
    user: impl Into<String> + Clone,
) -> StdResult<()> {
    let get_admin_auth_req: RouterConfigResponse =
        RouterQueryMsg::GetConfig {}.query(querier, contract)?;
    let admin_auth = get_admin_auth_req.config.admin_auth;
    validate_admin(
        querier,
        AdminPermissions::OraclesPriceBot,
        user,
        &admin_auth,
    )
}

/// Gets the admin auth contract from the router and uses it to check if the user is an admin for the router.
pub fn require_admin_or_bot(
    contract: &Contract,
    querier: &QuerierWrapper,
    user: impl Into<String> + Clone,
) -> StdResult<()> {
    let get_admin_auth_req: RouterConfigResponse =
        RouterQueryMsg::GetConfig {}.query(querier, contract)?;
    let admin_auth = get_admin_auth_req.config.admin_auth;
    let is_admin = validate_admin(
        querier,
        AdminPermissions::OraclesAdmin,
        user.clone(),
        &admin_auth,
    );
    let is_bot = validate_admin(
        querier,
        AdminPermissions::OraclesPriceBot,
        user,
        &admin_auth,
    );
    if is_admin.is_err() && is_bot.is_err() {
        Err(StdError::generic_err("User is not an admin or bot."))
    } else {
        Ok(())
    }
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
