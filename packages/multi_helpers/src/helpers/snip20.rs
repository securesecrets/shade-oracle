use super::*;
use shade_multi_test::multi::snip20::Snip20;
use shade_protocol::snip20::*;

create_test_helper!(Snip20Helper);

impl Snip20Helper {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        user: &User,
        app: &mut App,
        name: &str,
        symbol: &str,
        decimals: u8,
        admin: &Addr,
        initial_balances: &Option<Vec<InitialBalance>>,
        prng_seed: &Binary,
        label: &str,
    ) -> Self {
        let config = InitConfig {
            public_total_supply: Some(true),
            enable_deposit: None,
            enable_redeem: None,
            enable_mint: Some(true),
            enable_burn: Some(true),
            enable_transfer: Some(true),
        };

        let msg = InstantiateMsg {
            name: name.to_owned(),
            admin: Some(admin.to_string()),
            symbol: symbol.to_owned(),
            decimals,
            initial_balances: initial_balances.clone(),
            prng_seed: prng_seed.clone(),
            config: Some(config),
            query_auth: None,
        };

        let contract = user.init(app, &msg, Snip20::default(), label).unwrap();

        Snip20Helper(contract)
    }

    pub fn add_minters(&self, app: &mut App, sender: &User, minters: Vec<String>) {
        sender
            .exec(
                app,
                &ExecuteMsg::AddMinters {
                    minters,
                    padding: None,
                },
                &self.0,
            )
            .unwrap();
    }

    pub fn mint(
        &self,
        sender: &User,
        app: &mut App,
        recipient: &str,
        amount: impl Into<Uint128> + Copy,
    ) {
        let msg = ExecuteMsg::Mint {
            recipient: recipient.to_owned(),
            amount: amount.into(),
            memo: None,
            padding: None,
        };
        sender.exec(app, &msg, &self.0).unwrap();
    }

    pub fn set_viewing_key(&self, user: &User, app: &mut App, viewing_key: impl Into<String>) {
        let msg = ExecuteMsg::SetViewingKey {
            key: viewing_key.into(),
            padding: None,
        };
        user.exec(app, &msg, &self.0).unwrap();
    }

    /// Sets the viewing key for a slice for users and asserts that it can be used to query their balance.
    pub fn set_viewing_keys(
        &self,
        users: &[&User],
        app: &mut App,
        viewing_key: impl Into<String> + Copy,
    ) {
        let msg = ExecuteMsg::SetViewingKey {
            key: viewing_key.into(),
            padding: None,
        };
        for user in users {
            user.exec(app, &msg, &self.0).unwrap();
            assert!(User::query::<QueryAnswer>(
                app,
                &QueryMsg::Balance {
                    address: user.str(),
                    key: viewing_key.into(),
                },
                &self.0,
            )
            .is_ok())
        }
    }

    pub fn burn(&self, app: &mut App, user: &User, amount: impl Into<Uint128> + Copy) {
        user.exec(
            app,
            &ExecuteMsg::Burn {
                amount: amount.into(),
                memo: None,
                padding: None,
            },
            &self.0,
        )
        .unwrap();
    }

    pub fn get_balance(&self, app: &App, user: &str, viewing_key: impl Into<String>) -> Uint128 {
        let resp = User::query(
            app,
            &QueryMsg::Balance {
                address: user.to_string(),
                key: viewing_key.into(),
            },
            &self.0,
        )
        .unwrap();
        match resp {
            QueryAnswer::Balance { amount } => amount,
            _ => Uint128::zero(),
        }
    }

    pub fn send(
        &self,
        sender: &User,
        app: &mut App,
        recipient: impl Into<String>,
        amount: impl Into<Uint128> + Copy,
        msg: Option<Binary>,
    ) -> AppResult {
        sender.exec(
            app,
            &ExecuteMsg::Send {
                recipient: recipient.into(),
                recipient_code_hash: None,
                amount: amount.into(),
                msg,
                memo: None,
                padding: None,
            },
            &self.0,
        )
    }

    pub fn transfer(
        &self,
        sender: &User,
        app: &mut App,
        recipient: impl Into<String>,
        amount: impl Into<Uint128> + Copy,
    ) -> AppResult {
        sender.exec(
            app,
            &ExecuteMsg::Transfer {
                recipient: recipient.into(),
                amount: amount.into(),
                memo: None,
                padding: None,
            },
            &self.0,
        )
    }

    pub fn assert_user_balance(
        &self,
        user: &User,
        app: &App,
        viewing_key: &str,
        expected: impl Into<Uint128>,
    ) {
        let expected: Uint128 = expected.into();
        let actual = self.get_balance(app, &user.str(), viewing_key);
        assert_eq!(expected, actual);
    }

    pub fn assert_contract_balance(
        &self,
        contract: &ContractInfo,
        app: &App,
        viewing_key: &str,
        expected: impl Into<Uint128>,
    ) {
        let expected: Uint128 = expected.into();
        let actual = self.get_balance(app, contract.address.as_str(), viewing_key);
        assert_eq!(expected, actual);
    }
}
