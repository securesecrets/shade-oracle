use shade_oracles::{common as common_oracles, router};
use shade_oracles::{
    common::types::Contract,
    ensemble::ContractEnsemble,
    ensemble_helpers::EnsembleContract,
    ensemble_new, ensemblify,
    scrt::{ContractInstantiationInfo, HumanAddr, StdResult},
};
use std::{cell::RefCell, rc::Rc};

ensemblify!(
    OracleRouterHarness,
    ::oracle_router::contract::init,
    ::oracle_router::contract::handle,
    ::oracle_router::contract::query,
    OracleRouter
);

impl OracleRouter {
    ensemble_new!(OracleRouter, router::InitMsg);

    pub fn query_config(&self) -> StdResult<router::ConfigResponse> {
        self.query(&router::QueryMsg::GetOwner {})
    }

    pub fn query_price(&self, key: String) -> StdResult<common_oracles::PriceResponse> {
        self.query(&router::QueryMsg::GetPrice { key })
    }

    pub fn query_oracle(&self, key: String) -> StdResult<router::OracleResponse> {
        self.query(&router::QueryMsg::GetOracle { key })
    }

    pub fn change_admin(&self, new_owner: String, sender_key: Option<&str>) -> StdResult<()> {
        let msg = router::HandleMsg::ChangeOwner { new_owner };
        self.handle(&msg, sender_key)
    }

    pub fn update_registry(
        &self,
        operation: router::RegistryOperation,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        let msg = router::HandleMsg::UpdateRegistry { operation };
        self.handle(&msg, sender_key)
    }

    pub fn batch_update_registry(
        &self,
        operations: Vec<router::RegistryOperation>,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        let msg = router::HandleMsg::BatchUpdateRegistry { operations };
        self.handle(&msg, sender_key)
    }
}
