use cosmwasm_std::{Addr, ContractInfo};
use shade_protocol::{
    admin::ExecuteMsg,
    multi_test::{App, AppResponse},
    utils::ExecuteCallback,
    AnyResult,
};

pub struct AdminAuthHelper(pub ContractInfo);

impl AdminAuthHelper {
    pub fn update_registry(
        &self,
        sender: &Addr,
        app: &mut App,
        action: shade_protocol::admin::RegistryAction,
    ) -> AnyResult<AppResponse> {
        ExecuteMsg::UpdateRegistry { action }.test_exec(&self.0, app, sender.clone(), &[])
    }
    pub fn grant_access(
        &self,
        sender: &Addr,
        app: &mut App,
        user: String,
        permissions: Vec<String>,
    ) {
        let action = shade_protocol::admin::RegistryAction::GrantAccess { permissions, user };
        self.update_registry(sender, app, action).unwrap();
    }
    pub fn register_admin(&self, sender: &Addr, app: &mut App, user: String) {
        let action = shade_protocol::admin::RegistryAction::RegisterAdmin { user };
        self.update_registry(sender, app, action).unwrap();
    }
    pub fn revoke_access(
        &self,
        sender: &Addr,
        app: &mut App,
        user: String,
        permissions: Vec<String>,
    ) {
        let action = shade_protocol::admin::RegistryAction::RevokeAccess { permissions, user };
        self.update_registry(sender, app, action).unwrap();
    }
}
