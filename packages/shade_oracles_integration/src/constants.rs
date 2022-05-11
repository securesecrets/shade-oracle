// Math
pub const DECIMAL_FACTOR: u128 = 10u128.pow(6);

// Smart contracts
pub const STORE_GAS: &str = "10000000";
pub const GAS: &str = "800000";
pub const VIEW_KEY: &str = "password";

pub const ORACLE_ROUTER_FILE: &str = "../../compiled/oracle_router.wasm.gz";
pub const MOCK_BAND_FILE: &str = "../../compiled/mock_band.wasm.gz";
pub const PROXY_BAND_ORACLE_FILE: &str = "../../compiled/proxy_band_oracle.wasm.gz";
pub const SIENNASWAP_LP_SPOT_ORACLE_FILE: &str = "../../compiled/siennaswap_lp_spot_oracle.wasm.gz";
pub const SHADE_STAKING_DERIVATIVE_ORACLE_FILE: &str = "../../compiled/shade_staking_derivative_oracle.wasm.gz";
pub const EARN_V1_ORACLE_FILE: &str = "../../compiled/earn_v1_oracle.wasm.gz";


// Default executer & admin address for testing
pub const USER_A_KEY: &str = "a";
pub const USER_B_KEY: &str = "b";
pub const USER_C_KEY: &str = "c";
pub const USER_D_KEY: &str = "d";
pub const BACKEND: &str = "test";