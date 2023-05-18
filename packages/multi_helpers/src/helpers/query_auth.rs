use shade_toolkit::{implement_testable, interfaces::query_auth::{InstantiateMsg, ExecuteMsg}};

use super::*;
use ::query_auth as query_auth_crate;

implement_testable!(QueryAuth, query_auth_crate);
create_test_helper!(QueryAuthHelper);

impl QueryAuthHelper {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        user: &User,
        app: &mut App,
        label: &str,
        admin_auth: ContractInfo,
        prng_seed: Binary,
    ) -> Self {
        QueryAuthHelper(
            user.init(
                app,
                &InstantiateMsg {
                    admin_auth: admin_auth.into(),
                    prng_seed,
                },
                QueryAuth::default(),
                label,
            )
            .unwrap(),
        )
    }

    pub fn revoke_key(&self, user: &User, app: &mut App, key: &str, _admin: &Addr) {
        user.exec(
            app,
            &ExecuteMsg::BlockPermitKey {
                key: key.to_string(),
                padding: None,
            },
            &self.0,
        )
        .unwrap();
    }
}
