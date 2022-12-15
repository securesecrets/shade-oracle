use crate::multi::{MockBand, OracleRouter};
use shade_multi_test::multi::admin::init_admin_auth;
use shade_oracles::common::{PriceResponse, PricesResponse};
use shade_oracles::core::Query;
use shade_oracles::interfaces::router::msg::QueryMsg;
use shade_oracles::interfaces::router::registry::{RegistryOperation, UpdateConfig};
use shade_oracles::interfaces::{
    band::{self},
    router::{self},
};
use shade_protocol::admin::ExecuteMsg as AdminAuthExecuteMsg;
use shade_protocol::c_std::Decimal256;
use shade_protocol::multi_test::AppResponse;
use shade_protocol::{
    c_std::{Addr, ContractInfo, StdResult, Uint128, Uint256},
    multi_test::App,
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable},
    AnyResult, Contract,
};
use std::collections::HashMap;

pub struct AdminAuthHelper(pub ContractInfo);
impl AdminAuthHelper {
    pub fn update_registry(
        &self,
        sender: &Addr,
        app: &mut App,
        action: shade_protocol::admin::RegistryAction,
    ) -> AnyResult<AppResponse> {
        AdminAuthExecuteMsg::UpdateRegistry { action }.test_exec(&self.0, app, sender.clone(), &[])
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
pub struct BandHelper(pub ContractInfo);

impl BandHelper {
    pub fn update_prices(
        &self,
        sender: &Addr,
        app: &mut App,
        prices: HashMap<String, Uint128>,
        last_updated_time: u64,
    ) {
        for (sym, price) in prices {
            band::ExecuteMsg::UpdateSymbolPrice {
                base_symbol: sym,
                quote_symbol: "USD".into(),
                rate: price,
                last_updated: Some(last_updated_time),
            }
            .test_exec(&self.0, app, sender.clone(), &[])
            .unwrap();
        }
    }
}

pub struct OracleRouterHelper(pub ContractInfo);

impl OracleRouterHelper {
    pub fn update_registry(
        &self,
        sender: &Addr,
        app: &mut App,
        operation: RegistryOperation,
    ) -> AnyResult<AppResponse> {
        router::msg::ExecuteMsg::UpdateRegistry { operation }.test_exec(
            &self.0,
            app,
            sender.clone(),
            &[],
        )
    }
    pub fn add_oracle(
        &self,
        sender: &Addr,
        app: &mut App,
        oracle: Contract,
        key: String,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::Add { oracle, key })
    }

    pub fn remove_oracle(
        &self,
        sender: &Addr,
        app: &mut App,
        key: String,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::Remove { key })
    }

    pub fn replace_oracle(
        &self,
        sender: &Addr,
        app: &mut App,
        oracle: Contract,
        key: String,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::Replace { oracle, key })
    }
    pub fn protect_key(
        &self,
        sender: &Addr,
        app: &mut App,
        key: String,
        deviation: Decimal256,
        initial_price: Uint256,
    ) -> AnyResult<AppResponse> {
        self.update_registry(
            sender,
            app,
            RegistryOperation::Protect {
                key,
                deviation,
                initial_price,
            },
        )
    }
    pub fn remove_key_protection(
        &self,
        sender: &Addr,
        app: &mut App,
        key: String,
        deviation: Option<Decimal256>,
        price: Option<Uint256>,
    ) -> AnyResult<AppResponse> {
        self.update_registry(
            sender,
            app,
            RegistryOperation::UpdateProtection {
                key,
                deviation,
                price,
            },
        )
    }
    pub fn update_key_protection(
        &self,
        sender: &Addr,
        app: &mut App,
        key: String,
    ) -> AnyResult<AppResponse> {
        self.update_registry(sender, app, RegistryOperation::RemoveProtection { key })
    }
    pub fn update_protected_keys(
        &self,
        sender: &Addr,
        app: &mut App,
        prices: Vec<(String, Uint256)>,
    ) -> AnyResult<AppResponse> {
        router::msg::ExecuteMsg::UpdateProtectedKeys { prices }.test_exec(
            &self.0,
            app,
            sender.clone(),
            &[],
        )
    }
    pub fn query_price(&self, app: &App, key: String) -> StdResult<PriceResponse> {
        QueryMsg::GetPrice { key }.test_query(&self.0, app)
    }
    pub fn query_prices(&self, app: &App, keys: Vec<String>) -> StdResult<PricesResponse> {
        QueryMsg::GetPrices { keys }.test_query(&self.0, app)
    }
}

pub struct OracleCore {
    pub band: BandHelper,
    pub router: OracleRouterHelper,
    pub admin_auth: ContractInfo,
    pub superadmin: Addr,
}

impl OracleCore {
    pub fn new(
        band: BandHelper,
        router: OracleRouterHelper,
        admin_auth: ContractInfo,
        superadmin: Addr,
    ) -> Self {
        OracleCore {
            band,
            router,
            admin_auth,
            superadmin,
        }
    }
    /// Initializes the core dependencies for testing all oracles which are
    /// band, proxy band, router, and the admin auth contract. Then, it updates the prices in band
    /// based off the prices argument with them being quoted in "USD".
    pub fn setup(
        app: &mut App,
        admin: &Addr,
        prices: HashMap<String, Uint128>,
        band: Option<ContractInfo>,
        oracle_router: Option<ContractInfo>,
        admin_auth: Option<ContractInfo>,
    ) -> AnyResult<Self> {
        let quote_symbol = "USD".to_string();
        let superadmin = admin.clone();

        let admin_auth = admin_auth.unwrap_or_else(|| init_admin_auth(app, admin));
        let mut initial_prices = vec![];
        // Configure mock band prices
        for (sym, price) in prices {
            initial_prices.push((sym, quote_symbol.clone(), price));
        }

        let band = band.unwrap_or_else(|| {
            band::InstantiateMsg { initial_prices }
                .test_init(MockBand::default(), app, admin.clone(), "band", &[])
                .unwrap()
        });

        let oracle_router = oracle_router.unwrap_or_else(|| {
            router::msg::InstantiateMsg {
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
        });

        router::msg::ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                admin_auth: None,
                band: None,
                quote_symbol: None,
            },
        }
        .test_exec(&oracle_router, app, admin.clone(), &[])
        .unwrap();

        Ok(OracleCore::new(
            BandHelper(band),
            OracleRouterHelper(oracle_router),
            admin_auth,
            superadmin,
        ))
    }

    pub fn update_prices(
        &self,
        app: &mut App,
        prices: HashMap<String, Uint128>,
        last_updated_time: u64,
    ) {
        self.band
            .update_prices(&self.superadmin, app, prices, last_updated_time);
    }

    pub fn add_oracle(&self, app: &mut App, oracle: Contract, key: String) {
        self.router
            .add_oracle(&self.superadmin, app, oracle, key)
            .unwrap();
    }

    pub fn remove_oracle(&self, app: &mut App, key: String) {
        self.router
            .remove_oracle(&self.superadmin, app, key)
            .unwrap();
    }

    pub fn replace_oracle(&self, app: &mut App, oracle: Contract, key: String) {
        self.router
            .replace_oracle(&self.superadmin, app, oracle, key)
            .unwrap();
    }
}
