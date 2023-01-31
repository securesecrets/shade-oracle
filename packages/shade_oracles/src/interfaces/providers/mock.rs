use super::*;

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_prices: Vec<(String, String, Uint128)>,
    pub admin_auth: RawContract,
    pub quote_symbol: Option<String>,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub struct Config {
    pub admin_auth: Contract,
    pub quote_symbol: String,
    pub enabled: bool,
}

pub type ConfigResponse = Config;

impl Config {
    pub fn require_admin(
        &self,
        querier: &QuerierWrapper,
        user: impl Into<String> + Clone,
    ) -> StdResult<()> {
        validate_admin(
            querier,
            AdminPermissions::OraclesAdmin,
            user,
            &self.admin_auth,
        )
    }

    pub fn require_admin_or_bot(
        &self,
        querier: &QuerierWrapper,
        user: impl Into<String> + Clone,
    ) -> StdResult<()> {
        let admin_auth = &self.admin_auth;
        let is_admin = validate_admin(
            querier,
            AdminPermissions::OraclesAdmin,
            user.clone(),
            admin_auth,
        );
        let is_bot = validate_admin(querier, AdminPermissions::OraclesPriceBot, user, admin_auth);
        if is_admin.is_err() && is_bot.is_err() {
            Err(StdError::generic_err("User is not an admin or bot."))
        } else {
            Ok(())
        }
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    SetStatus(bool),
    UpdateConfig {
        admin_auth: Option<RawContract>,
        quote_symbol: Option<String>,
    },
    SetPrice(MockPrice),
    SetPrices(Vec<MockPrice>),
}

#[cw_serde]
pub struct MockPrice {
    pub base_symbol: String,
    pub quote_symbol: String,
    pub rate: Uint128,
    pub last_updated: Option<u64>,
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum ExecuteAnswer {
    SetPrice { status: ResponseStatus },
    SetPrices { status: ResponseStatus },
}
