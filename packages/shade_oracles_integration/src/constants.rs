// Math
pub const DECIMAL_FACTOR: u128 = 10u128.pow(6);

// Smart contracts
pub const STORE_GAS: &str = "3000000";
pub const GAS: &str = "1500000";
pub const VIEW_KEY: &str = "password";

pub const ORACLE_ROUTER_FILE: &str = "../../../res/bonds-0.10/compiled/oracle_router.wasm.gz";
pub const MOCK_BAND_FILE: &str = "../../../res/bonds-0.10/compiled/mock_band.wasm.gz";
pub const PROXY_BAND_ORACLE_FILE: &str =
    "../../../res/bonds-0.10/compiled/proxy_band_oracle.wasm.gz";
pub const SIENNASWAP_LP_SPOT_ORACLE_FILE: &str =
    "../../../res/bonds-0.10/compiled/siennaswap_lp_spot_oracle.wasm.gz";
pub const SHADE_STAKING_DERIVATIVE_ORACLE_FILE: &str =
    "../../../res/bonds-0.10/compiled/shade_staking_derivative_oracle.wasm.gz";
pub const EARN_V1_ORACLE_FILE: &str = "../../../res/bonds-0.10/compiled/earn_v1_oracle.wasm.gz";
pub const INDEX_ORACLE_FILE: &str = "../../../res/bonds-0.10/compiled/index_oracle.wasm.gz";
pub const SIENNA_MARKET_ORACLE_FILE: &str =
    "../../../res/bonds-0.10/compiled/siennaswap_market_oracle.wasm.gz";

// Default executer & admin address for testing
pub const USER_A_KEY: &str = "a";
pub const USER_B_KEY: &str = "b";
pub const USER_C_KEY: &str = "c";
pub const USER_D_KEY: &str = "d";
pub const HOOMP_KEY: &str = "hoomp";
pub const DEPLOY_KEY: &str = "shade-deploy";
pub const BACKEND: &str = "test";

pub mod keys {

    pub const STKD_SCRT_SHD_LP: &str = "stkd-SCRT/SHD SiennaSwap LP";
    pub const STKD_SCRT: &str = "stkd-SCRT";
    pub const SHD_SSCRT_LP: &str = "SHD/SSCRT SiennaSwap LP";
    pub const STKD_SCRT_SCRT_LP: &str = "stkd-SCRT/SCRT SiennaSwap LP";
    pub const SSCRT: &str = "SSCRT";
    pub const SCRT: &str = "SCRT";
    pub const SHD: &str = "SHD";
}

pub mod mainnet {

    pub const OSMO_TOKEN_SYM: &str = "SOSMO";
    pub const ATOM_TOKEN_SYM: &str = "SATOM";
    pub const OSMO_TOKEN_NAME: &str = "Secret OSMO";
    pub const ATOM_TOKEN_NAME: &str = "Secret ATOM";
    pub const SSCRT_TOKEN_NAME: &str = "ssecret";
    pub const STKD_SCRT_TOKEN_NAME: &str = "Staked SCRT Derivative (Shade)";
    pub const SHADE_TOKEN_NAME: &str = "Shade";

    pub mod sienna {

        pub const STKD_SCRT_SCRT_POOL: &str = "secret155ycxc247tmhwwzlzalakwrerde8mplhluhjct";
        pub const STKD_SCRT_SCRT_TOKEN_NAME: &str = "SiennaSwap Liquidity Provider (LP) token for secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek-secret1k6u0cy4feepm6pehnz804zmwakuwdapm69tuc4";
        pub const STKD_SCRT_SCRT_POOL_HASH: &str =
            "33EAC42C44EE69ACFE1F56CE7B14FE009A7B611E86F275D7AF2D32DD0D33D5A9";

        pub const STKD_SCRT_SHD_POOL: &str = "secret19frhcpqr3e3g723d484hlcvf0tkumnr76eg8qc";
        pub const STKD_SCRT_SHD_TOKEN_NAME: &str = "SiennaSwap Liquidity Provider (LP) token for secret1k6u0cy4feepm6pehnz804zmwakuwdapm69tuc4-secret1qfql357amn448duf5gvp9gr48sxx9tsnhupu3d";
        pub const STKD_SCRT_SHD_POOL_HASH: &str =
            "33EAC42C44EE69ACFE1F56CE7B14FE009A7B611E86F275D7AF2D32DD0D33D5A9";

        pub const SHD_SSCRT_POOL: &str = "secret1drm0dwvewjyy0rhrrw485q4f5dnfm6j25zgfe5";
        pub const SHD_SSCRT_TOKEN_NAME: &str = "SiennaSwap Liquidity Provider (LP) token for secret1qfql357amn448duf5gvp9gr48sxx9tsnhupu3d-secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek";
        pub const SHD_SSCRT_POOL_HASH: &str =
            "33EAC42C44EE69ACFE1F56CE7B14FE009A7B611E86F275D7AF2D32DD0D33D5A9";
    }

    pub const SHD: &str = "secret1qfql357amn448duf5gvp9gr48sxx9tsnhupu3d";

    // TODO
    pub const BAND: &str = "secret1ezamax2vrhjpy92fnujlpwfj2dpredaafss47k";
    pub const BAND_HASH: &str = "00230665FA8DC8BB3706567CF0A61F282EDC34D2F7DF56192B2891FD9CD27B06";

    pub const STKD_SCRT: &str = "secret1k6u0cy4feepm6pehnz804zmwakuwdapm69tuc4";
    pub const STKD_SCRT_HASH: &str =
        "F6BE719B3C6FEB498D3554CA0398EB6B7E7DB262ACB33F84A8F12106DA6BBB09";

    pub const ADMIN_AUTH: &str = "secret1lrtayuylgdgdc9ekqw7ln7yhujapy9dg7x5qd0";
    pub const ADMIN_AUTH_HASH: &str =
        "8dd3d519e7a7a05260688d1f4b39fa3d1d76d7692de8c9ae579d6c8d58c5f7dd";
}

pub mod local {
    pub const ADMIN_AUTH: &str = "secret1pg606jw68d9mnh9czrgm7celc3rq9x5w8duhhr";
    pub const ADMIN_AUTH_HASH: &str =
        "8dd3d519e7a7a05260688d1f4b39fa3d1d76d7692de8c9ae579d6c8d58c5f7dd";
    pub const STKD_SCRT: &str = "secret10d6vwc2ckdszk6z6u6hafy9mrhssmvfhaw7cwn";
    pub const STKD_SCRT_HASH: &str =
        "680fbb3c8f8eb1c920da13d857daaedaa46ab8f9a8e26e892bb18a16985ec29e";
}

pub mod testnet {

    pub const BAND: &str = "secret14swdnnllsfvtnvwmtvnvcj2zu0njsl9cdkk5xp";
    pub const BAND_HASH: &str = "00230665FA8DC8BB3706567CF0A61F282EDC34D2F7DF56192B2891FD9CD27B06";
    pub const STKD_SCRT: &str = "secret15kuyl5e74fp3xqfzctn3gff0lxw44k4utqpzyw";
    pub const STKD_SCRT_HASH: &str =
        "F6BE719B3C6FEB498D3554CA0398EB6B7E7DB262ACB33F84A8F12106DA6BBB09";
    pub const SIENNA_STKD_SCRT_SCRT_POOL: &str = "secret132xpyzj9eussjgukx9srwseh7mgm3d2g4d34pt";
    pub const SIENNA_STKD_SCRT_SCRT_POOL_HASH: &str =
        "33EAC42C44EE69ACFE1F56CE7B14FE009A7B611E86F275D7AF2D32DD0D33D5A9";

    pub const SIENNA_STKD_SCRT_SHD_POOL: &str = "secret1y25r7qjnllktj8cr60pf6avuer3jxqqulpjapt";
    pub const SIENNA_STKD_SCRT_SHD_POOL_HASH: &str =
        "33eac42c44ee69acfe1f56ce7b14fe009a7b611e86f275d7af2d32dd0d33d5a9";

    pub const SIENNA_SHD_SSCRT_POOL: &str = "secret1pak8feexy97myp22pjkxmsp5p8dmlkp4mkfxsl";
    pub const SIENNA_SHD_SSCRT_POOL_HASH: &str =
        "33eac42c44ee69acfe1f56ce7b14fe009a7b611e86f275d7af2d32dd0d33d5a9";
    pub const ADMIN_AUTH: &str = "secret15l3p2sc6v22levjgwk3d856gljpaary28jefzt";
    pub const ADMIN_AUTH_HASH: &str =
        "1BFA6A48B1B6FCCDC823A80AEA3DAC198D91BAF5A62F5858AE4C6BC7B4CA5ABD";
}
