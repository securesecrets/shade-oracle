pub mod error;
pub mod msg;
pub mod registry;

pub mod querier {
    use super::msg::ConfigResponse;
    use super::msg::*;
    use cosmwasm_std::{QuerierWrapper, StdResult};
    use shade_protocol::{utils::Query, Contract};

    pub fn get_admin_auth(
        router: &Contract,
        querier: &QuerierWrapper,
    ) -> StdResult<ConfigResponse> {
        let resp: ConfigResponse = QueryMsg::GetConfig {}.query(querier, router)?;
        Ok(resp)
    }
}
