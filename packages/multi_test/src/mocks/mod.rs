use super::*;

pub use shade_stkd_scrt::*;
pub use shadeswap::*;
pub use sienna_pair::*;

macro_rules! create_harness {
    ($mod_name:ident, $name:ident, $contract_path:path, $contract:ident) => {
        pub use $mod_name::*;
        mod $mod_name {
            use $contract_path;
            multi_derive::implement_multi!($name, $contract);
        }
    };
}

mod shade_stkd_scrt {
    create_test_helper!(MockShadeStkdScrtHelper);

    impl MockShadeStkdScrtHelper {}

    create_harness!(
        harness,
        MockShadeStkdScrt,
        oracle_mocks::shade_stkd_scrt,
        shade_stkd_scrt
    );
}

mod shadeswap {

    create_harness!(
        harness,
        MockShadeswapPair,
        oracle_mocks::shadeswap_pair,
        shadeswap_pair
    );
}
mod sienna_pair {

    create_harness!(
        harness,
        MockSiennaPair,
        oracle_mocks::sienna_pair,
        sienna_pair
    );
}
