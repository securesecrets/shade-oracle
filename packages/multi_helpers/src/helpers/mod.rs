use super::*;
use cosmwasm_std::{Addr, StdError};
use serde::de::DeserializeOwned;
use shade_toolkit::{
    multi::{nanoid::nanoid, AnyResult, MultiTestable, Tester},
    multi_test::{App, AppResponse},
};
use std::{
    fmt::{Debug, Display},
    panic::{self, AssertUnwindSafe},
};

pub mod admin_auth;
pub mod query_auth;
pub mod snip20;

pub type AppResult = AnyResult<AppResponse>;

/// Alphabet used by nanoid for generating addresses. Has the characters '-' and '_' removed and upper case characters so that generated address is normalized.
/// Very low probability of collisions - https://alex7kom.github.io/nano-nanoid-cc.
/// Using address length of 40.
pub const SAFE_NANOID: [char; 36] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
pub const SAFE_NANOID_LENGTH: usize = 40;

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
impl Tester for User {
    fn addr(&self) -> Addr {
        self.address.clone()
    }
}

impl User {
    pub fn new(address: impl Into<String>) -> Self {
        let address = Addr::unchecked(address);
        User { address }
    }
    pub fn str(&self) -> String {
        self.address.to_string()
    }
    pub fn gen() -> User {
        User::new(format!(
            "secret{}",
            nanoid!(SAFE_NANOID_LENGTH, &SAFE_NANOID)
        ))
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
