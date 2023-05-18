use super::*;
use cosmwasm_std::{Addr, StdError};
use serde::de::DeserializeOwned;
use shade_toolkit::{
    multi_test::{App, AppResponse},
    MultiTestable,
    AnyResult,
};
use std::{
    fmt::{Debug, Display},
    panic::{self, AssertUnwindSafe},
};

pub mod admin_auth;
pub mod query_auth;
pub mod snip20;

pub type AppResult = AnyResult<AppResponse>;

pub struct Asserter;

impl Asserter {
    /// Assert that the result of code that is unwind safe is an error.
    pub fn unwind_err(hook: impl FnOnce()) {
        let res = panic::catch_unwind(AssertUnwindSafe(hook));
        let is_err = res.is_err();
        assert!(is_err);
    }

    pub fn app_err(hook: impl FnOnce() -> AppResult) {
        assert!(hook().is_err())
    }

    pub fn std_err<T: DeserializeOwned>(hook: impl FnOnce() -> StdResult<T>) {
        assert!(hook().is_err())
    }

    pub fn get_std_err_msg(error: anyhow::Error) -> String {
        let source: &StdError = error.downcast_ref().unwrap();
        match source {
            StdError::GenericErr { msg, .. } => msg.to_string(),
            _ => panic!("Expected StdError::GenericErr"),
        }
    }

    /// Assert that the AppResult hook returns an error. Can only handle errors bubbled up from 1 depth.
    pub fn err<E>(hook: impl FnOnce() -> AppResult, error: &E)
    where
        E: Display + Debug + Send + Sync + PartialEq + 'static,
    {
        let err = hook().unwrap_err();

        let source: &E = err.downcast_ref().unwrap();
        assert_eq!(source, error);
    }

    pub fn equal_vecs<T>(a: &[T], b: &[T])
    where
        T: Ord + Debug + Clone,
    {
        let mut a = a.to_vec();
        let mut b = b.to_vec();
        a.sort();
        b.sort();
        assert_eq!(a, b);
    }
}

#[derive(Clone)]
/// Wraps an Addr and provides helper methods for testing.
pub struct User {
    pub address: Addr,
}
impl User {
    pub fn new(address: impl Into<String>) -> Self {
        let address = Addr::unchecked(address);
        User { address }
    }

    pub fn addr(&self) -> Addr {
        self.address.clone()
    }
    pub fn str(&self) -> String {
        self.address.to_string()
    }
    pub fn init(
        &self,
        app: &mut App,
        msg: &impl InstantiateCallback,
        testable: impl MultiTestable,
        label: &str,
    ) -> AnyResult<ContractInfo> {
        msg.test_init(testable, app, self.address.clone(), label, &[])
    }
    pub fn init_with_funds(
        &self,
        app: &mut App,
        msg: &impl InstantiateCallback,
        testable: impl MultiTestable,
        label: &str,
        send_funds: &[Coin],
    ) -> AnyResult<ContractInfo> {
        msg.test_init(testable, app, self.address.clone(), label, send_funds)
    }
    pub fn exec(
        &self,
        app: &mut App,
        msg: &(impl ExecuteCallback + std::fmt::Debug),
        contract: &ContractInfo,
    ) -> AnyResult<AppResponse> {
        msg.test_exec(contract, app, self.address.clone(), &[])
    }
    pub fn exec_with_funds(
        &self,
        app: &mut App,
        msg: &(impl ExecuteCallback + std::fmt::Debug),
        contract: &ContractInfo,
        send_funds: &[Coin],
    ) -> AnyResult<AppResponse> {
        msg.test_exec(contract, app, self.address.clone(), send_funds)
    }
}

impl From<Addr> for User {
    fn from(a: Addr) -> Self {
        User { address: a }
    }
}

impl<'a> From<&'a Addr> for User {
    fn from(a: &'a Addr) -> Self {
        User { address: a.clone() }
    }
}

impl From<String> for User {
    fn from(s: String) -> Self {
        User {
            address: Addr::unchecked(s),
        }
    }
}

impl From<&String> for User {
    fn from(s: &String) -> Self {
        User {
            address: Addr::unchecked(s.clone()),
        }
    }
}

impl<'a> From<&'a str> for User {
    fn from(s: &'a str) -> Self {
        User {
            address: Addr::unchecked(s),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<Addr> for User {
    fn into(self) -> Addr {
        self.address
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for User {
    fn into(self) -> String {
        self.address.to_string()
    }
}
