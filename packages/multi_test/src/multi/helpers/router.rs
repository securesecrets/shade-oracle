use shade_oracles::interfaces::router::msg::*;

use super::*;

pub struct OracleRouterHelper(pub ContractInfo);

impl OracleRouterHelper {
    pub fn init(user: &User, app: &mut App, admin_auth: &C) -> Self {
        InstantiateMsg {
            admin_auth: admin_auth.clone().into(),
            band: band.clone().into(),
            quote_symbol: quote_symbol.clone(),
        }
        .test_init(
            OracleRouter::default(),
            app,
            admin.clone(),
            "oracle_router",
            &[],
        )
        .unwrap()
    }
    pub fn update_registry(
        &self,
        sender: &Addr,
        app: &mut App,
        operation: RegistryOperation,
    ) -> AnyResult<AppResponse> {
        ExecuteMsg::UpdateRegistry(operation).test_exec(&self.0, app, sender.clone(), &[])
    }
    pub fn set_keys(
        &self,
        sender: &Addr,
        app: &mut App,
        oracle: Contract,
        keys: Vec<String>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(
            sender,
            app,
            RegistryOperation::SetKeys {
                oracle: oracle.into(),
                keys,
            },
        )
    }

    pub fn remove_keys(
        &self,
        sender: &Addr,
        app: &mut App,
        keys: Vec<String>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::RemoveKeys { keys })
    }

    pub fn set_protection(
        &self,
        sender: &Addr,
        app: &mut App,
        infos: Vec<router::registry::ProtectedKeyInfo>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::SetProtection { infos })
    }
    pub fn remove_key_protections(
        &self,
        sender: &Addr,
        app: &mut App,
        keys: Vec<String>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::RemoveProtection { keys })
    }
    pub fn update_protected_keys(
        &self,
        sender: &Addr,
        app: &mut App,
        updates: Vec<(String, Uint256)>,
    ) -> AnyResult<AppResponse> {
        ExecuteMsg::UpdateProtectedKeys(updates).test_exec(&self.0, app, sender.clone(), &[])
    }
    pub fn query_price(&self, app: &App, key: String) -> StdResult<PriceResponse> {
        QueryMsg::GetPrice { key }.test_query(&self.0, app)
    }
    pub fn query_prices(&self, app: &App, keys: Vec<String>) -> StdResult<PricesResponse> {
        QueryMsg::GetPrices { keys }.test_query(&self.0, app)
    }
}
