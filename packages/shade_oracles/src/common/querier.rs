use crate::interfaces::{
    common::{OracleQuery, PriceResponse, PricesResponse},
    router::msg::{ConfigResponse as RouterConfigResponse, QueryMsg as RouterQueryMsg},
};
use cosmwasm_std::{QuerierWrapper, StdError, StdResult};
use shade_toolkit::{
    interfaces::admin_auth::{validate_admin, AdminPermissions},
    Contract, Query,
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
