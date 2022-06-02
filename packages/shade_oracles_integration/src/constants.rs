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
pub const SHADE_STAKING_DERIVATIVE_ORACLE_FILE: &str =
    "../../compiled/shade_staking_derivative_oracle.wasm.gz";
pub const EARN_V1_ORACLE_FILE: &str = "../../compiled/earn_v1_oracle.wasm.gz";
pub const INDEX_ORACLE_FILE: &str = "../../compiled/index_oracle.wasm.gz";

// Default executer & admin address for testing
pub const USER_A_KEY: &str = "a";
pub const USER_B_KEY: &str = "b";
pub const USER_C_KEY: &str = "c";
pub const USER_D_KEY: &str = "d";
pub const HOOMP_KEY: &str = "hoomp";
pub const BACKEND: &str = "test";

pub mod testnet {
    pub const BAND: &str = "secret14swdnnllsfvtnvwmtvnvcj2zu0njsl9cdkk5xp";
    pub const BAND_HASH: &str = "00230665FA8DC8BB3706567CF0A61F282EDC34D2F7DF56192B2891FD9CD27B06";
    pub const STKD_SCRT: &str = "secret15kuyl5e74fp3xqfzctn3gff0lxw44k4utqpzyw";
    pub const STKD_SCRT_HASH: &str =
        "F6BE719B3C6FEB498D3554CA0398EB6B7E7DB262ACB33F84A8F12106DA6BBB09";
    pub const STKD_SCRT_SCRT_POOL: &str = "secret132xpyzj9eussjgukx9srwseh7mgm3d2g4d34pt";
    pub const STKD_SCRT_SCRT_POOL_HASH: &str =
        "33EAC42C44EE69ACFE1F56CE7B14FE009A7B611E86F275D7AF2D32DD0D33D5A9";
}
