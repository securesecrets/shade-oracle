use shade_toolkit::{implement_harness, interfaces::admin_auth::{InstantiateMsg, RegistryAction, ExecuteMsg}};
use super::*;

implement_harness!(Admin, admin);
create_test_helper!(AdminAuthHelper);

impl AdminAuthHelper {
    pub fn init(app: &mut App, sender: &User, superadmin: Option<Addr>) -> Self {
        let superadmin = superadmin.unwrap_or_else(|| sender.addr());
        let msg = InstantiateMsg {
            super_admin: Some(superadmin.to_string()),
        };
        Self(
            sender
                .init(app, &msg, Admin::default(), "admin_auth")
                .unwrap(),
        )
    }
    pub fn update_registry(
        &self,
        sender: &User,
        app: &mut App,
        action: RegistryAction,
    ) -> AnyResult<AppResponse> {
        sender.exec(app, &ExecuteMsg::UpdateRegistry { action }, &self.0)
    }
    pub fn grant_access(
        &self,
        sender: &User,
        app: &mut App,
        user: String,
        permissions: Vec<String>,
    ) {
        let action = RegistryAction::GrantAccess { permissions, user };
        self.update_registry(sender, app, action).unwrap();
    }
    pub fn register_admin(&self, sender: &User, app: &mut App, user: String) {
        let action = RegistryAction::RegisterAdmin { user };
        self.update_registry(sender, app, action).unwrap();
    }
    pub fn revoke_access(
        &self,
        sender: &User,
        app: &mut App,
        user: String,
        permissions: Vec<String>,
    ) {
        let action = RegistryAction::RevokeAccess { permissions, user };
        self.update_registry(sender, app, action).unwrap();
    }
}
