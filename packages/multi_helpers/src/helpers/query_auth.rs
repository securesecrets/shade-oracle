use super::*;
pub use shade_multi_test::multi::query_auth::QueryAuth;
use shade_protocol::{
    contract_interfaces::query_auth::{ExecuteMsg, InstantiateMsg},
    query_auth::{PermitData, QueryPermit},
    query_authentication::transaction::{PermitSignature, PubKey},
};

pub fn user_with_permit_1() -> Addr {
    Addr::unchecked("secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq")
}

pub fn get_permit<T>(data: T) -> QueryPermit
where
    T: Serialize,
{
    QueryPermit {
            params: PermitData {
                key: "key".to_string(),
                data: to_binary(&data).unwrap(),
            },
            signature: PermitSignature {
                pub_key: PubKey::new(
                    Binary::from_base64(
                        "A9NjbriiP7OXCpoTov9ox/35+h5k0y1K0qCY/B09YzAP"
                    ).unwrap()
                ),
                signature: Binary::from_base64(
                    "XRzykrPmMs0ZhksNXX+eU0TM21fYBZXZogr5wYZGGy11t2ntfySuQNQJEw6D4QKvPsiU9gYMsQ259dOzMZNAEg=="
                ).unwrap()
            },
            account_number: None,
            chain_id: Some(String::from("chain")),
            sequence: None,
            memo: None
        }
}

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
