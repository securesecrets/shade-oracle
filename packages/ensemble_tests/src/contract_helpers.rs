use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use shared_types::{
    asset::Contract,
    ensemble::{ContractEnsemble, MockEnv},
    scrt::{ContractInstantiationInfo, ContractLink, HumanAddr, StdResult},
};

use crate::constants::DEFAULT_ADMIN;

pub mod oracles;
pub mod token;

pub trait EnsembleContract {
    fn get_info(&self) -> ContractLink<HumanAddr>;
    fn as_contract(&self) -> Contract;
    fn get_ensemble(&self) -> RefMut<ContractEnsemble>;
    fn handle<Message: serde::Serialize>(
        &self,
        msg: &Message,
        sender_key: Option<&str>,
    ) -> StdResult<()> {
        let env = MockEnv::new(sender_key.unwrap_or(DEFAULT_ADMIN), self.get_info());
        self.get_ensemble().execute(msg, env)
    }
    fn init<Message: serde::Serialize>(
        msg: &Message,
        ensemble: &Rc<RefCell<ContractEnsemble>>,
        init_info: &ContractInstantiationInfo,
        address: &str,
        sender_key: Option<&str>,
    ) -> ContractLink<HumanAddr> {
        let env = MockEnv::new(
            sender_key.unwrap_or(DEFAULT_ADMIN),
            ContractLink {
                address: address.into(),
                code_hash: init_info.code_hash.clone(),
            },
        );
        let info = ensemble
            .borrow_mut()
            .instantiate(init_info.id, msg, env)
            .unwrap();
        info
    }
    fn query<Message: serde::Serialize, T: serde::Serialize + serde::de::DeserializeOwned>(
        &self,
        msg: &Message,
    ) -> StdResult<T> {
        self.get_ensemble()
            .query(self.get_info().address.clone(), msg)
    }
}

#[macro_export]
macro_rules! ensemblify {
    ($harness: ident, $init: path, $handle: path, $query: path, $element: ident) => {
        pub struct $harness;
        impl shared_types::ensemble::ContractHarness for $harness {
            fn init(
                &self,
                deps: &mut shared_types::ensemble::MockDeps,
                env: shared_types::scrt::Env,
                msg: shared_types::scrt::Binary,
            ) -> shared_types::scrt::StdResult<shared_types::scrt::InitResponse> {
                $init(deps, env, shared_types::scrt::from_binary(&msg)?)
            }

            fn handle(
                &self,
                deps: &mut shared_types::ensemble::MockDeps,
                env: shared_types::scrt::Env,
                msg: shared_types::scrt::Binary,
            ) -> shared_types::scrt::StdResult<shared_types::scrt::HandleResponse> {
                $handle(deps, env, shared_types::scrt::from_binary(&msg)?)
            }

            fn query(
                &self,
                deps: &shared_types::ensemble::MockDeps,
                msg: shared_types::scrt::Binary,
            ) -> shared_types::scrt::StdResult<shared_types::scrt::Binary> {
                $query(deps, shared_types::scrt::from_binary(&msg)?)
            }
        }
        pub struct $element {
            pub info: shared_types::scrt::ContractLink<HumanAddr>,
            pub ensemble: std::rc::Rc<std::cell::RefCell<shared_types::ensemble::ContractEnsemble>>,
        }
        impl crate::contract_helpers::EnsembleContract for $element {
            fn as_contract(&self) -> shared_types::asset::Contract {
                Contract::new(&self.get_info())
            }
            fn get_info(&self) -> shared_types::scrt::ContractLink<HumanAddr> {
                self.info.clone()
            }
            fn get_ensemble(&self) -> std::cell::RefMut<shared_types::ensemble::ContractEnsemble> {
                self.ensemble.borrow_mut()
            }
        }
    };
}

#[macro_export]
macro_rules! ensemble_new {
    ($struct_name: ident, $msg_struct: ty) => {
        pub fn new(
            msg: &$msg_struct,
            ensemble: &Rc<RefCell<ContractEnsemble>>,
            init_info: ContractInstantiationInfo,
            address: &str,
            account_key: Option<&str>,
        ) -> Self {
            let info = Self::init(&msg, ensemble, &init_info, address, account_key);
            $struct_name {
                info,
                ensemble: ensemble.clone(),
            }
        }
    };
}
