use super::*;

use better_secret_math::{
    core::{bankers_round, exp10, muldiv_fp},
    U256,
};
use serde::de::DeserializeOwned;

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

    /// Assert that the AppResult hook returns an error. Can only handle errors bubbled up from 1 depth.
    pub fn err<E>(hook: impl FnOnce() -> AppResult, error: &E)
    where
        E: Display + Debug + Send + Sync + PartialEq + 'static,
    {
        let err = hook().unwrap_err();

        let source: &E = err.downcast_ref().unwrap();
        assert_eq!(source, error);
    }

    /// Assert a and b are within error distance of one another
    /// a, b, and error must be normalized to 10^18
    pub fn close_u128(a: u128, b: u128, error: u128) {
        // Get absolute different of a and b
        let diff = a.abs_diff(b);
        // Ensure diff is within inputted margin of error
        let error_diff = if a < b {
            muldiv_fp(U256::from(a), exp10(18) + U256::from(error))
                .unwrap()
                .as_u128()
                - a
        } else {
            a - muldiv_fp(U256::from(a), exp10(18) - U256::from(error))
                .unwrap()
                .as_u128()
        };
        assert!(diff <= error_diff);
    }

    /// Employs bankers rounding on the (x - n)th decimal of actual where x is actual's decimal precision.
    ///
    /// So if x is 18, n is 1, it will perform bankers rounding to the 17th decimal and check if expected and actual are the same afterwards.
    pub fn close_u256(expected: impl Into<U256> + Copy, actual: impl Into<U256> + Copy, n: u8) {
        let actual: U256 = actual.into();
        let expected: U256 = expected.into();
        assert_eq!(expected, bankers_round(actual, n));
    }

    // Asserts that expected and actual are within 17 decimal precision of each other using bankers rounding on the actual value.
    pub fn bigint(expected: impl Into<U256> + Copy, actual: impl Into<U256> + Copy) {
        Self::close_u256(expected, actual, 1);
    }

    /// Asserts the actual value is equal to expected after truncating some amount of its decimals.
    pub fn close_uint256(expected: u128, actual: Uint256, decimals: u32) {
        assert_eq!(
            Uint256::from_u128(expected),
            actual / Uint256::from_u128(10u128.pow(decimals))
        );
    }

    /// Asserts the actual value is equal to expected after truncating some amount of its decimals.
    pub fn close_uint128(expected: u128, actual: Uint128, decimals: u32) {
        assert_eq!(
            Uint128::new(expected),
            actual / Uint128::new(10u128.pow(decimals))
        );
    }
}

#[derive(Clone)]
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
    pub fn query<T: DeserializeOwned>(
        app: &App,
        msg: &impl Query,
        contract: &ContractInfo,
    ) -> StdResult<T> {
        msg.test_query(contract, app)
    }
    pub fn init(
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
