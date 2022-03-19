use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared_types::{
    asset::{Contract, ResponseStatus},
    scrt::*,
    secret_toolkit::utils::Query,
    scrt_math::Uint256,
};

pub mod common {
    use super::*;
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        GetConfig {},
        GetPrice {},
    }

    // We define a custom struct for each query response
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct PriceResponse {
        pub rate: Uint128,
        pub last_updated_base: u64,
        pub last_updated_quote: u64,
    }

    pub fn query_price(contract: &Contract, querier: &impl Querier) -> StdResult<PriceResponse> {
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: HumanAddr(contract.address.clone()),
            callback_code_hash: contract.code_hash.clone(),
            msg: to_binary(&QueryMsg::GetPrice {})?,
        }))
    }
}

pub mod band {
    use super::*;
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {}

    #[derive(Serialize, Deserialize, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        UpdateSymbolPrice {
            base_symbol: String,
            quote_symbol: String,
            rate: Uint128,
            last_updated: Option<u64>,
        },
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleAnswer {
        UpdateSymbolPrice { status: ResponseStatus },
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum BandQuery {
        GetReferenceData {
            base_symbol: String,
            quote_symbol: String,
        },
        GetReferenceDataBulk {
            base_symbols: Vec<String>,
            quote_symbols: Vec<String>,
        },
    }

    impl Query for BandQuery {
        const BLOCK_SIZE: usize = 256;
    }

    #[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
    pub struct ReferenceData {
        pub rate: Uint128,
        pub last_updated_base: u64,
        pub last_updated_quote: u64,
    }
}

pub mod proxy_band_oracle {
    use super::*;
    // base_asset quoted in quote_asset, Ex: BTC (base) quoted in USD(quote)
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub owner: String,
        pub band: Contract,
        pub base_symbol: String,
        pub quote_symbol: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        UpdateConfig {
            owner: Option<String>,
            band: Option<Contract>,
            base_symbol: Option<String>,
            quote_symbol: Option<String>,
        },
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleAnswer {
        UpdateConfig { status: ResponseStatus },
    }

    // We define a custom struct for each query response
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct ConfigResponse {
        pub owner: String,
        pub band: Contract,
        pub base_symbol: String,
        pub quote_symbol: String,
    }
}

pub mod earn_v1_oracle {
    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct InitMsg {
        pub owner: String,
        pub deposit_token_oracle: Contract,
        pub strategy: Contract,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        UpdateConfig {
            owner: Option<String>,
            deposit_token_oracle: Option<Contract>,
            strategy: Option<Contract>,
        },
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleAnswer {
        UpdateConfig { status: ResponseStatus },
    }

    // We define a custom struct for each query response
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct ConfigResponse {
        pub owner: String,
        pub deposit_token_oracle: Contract,
        pub deposit_token: Contract,
        pub share_token: Contract,
        pub strategy: Contract,
    }
}

pub mod lp_oracle {
    use super::*;
    /// Oracle1 - contract for an oracle of asset 1
    ///
    /// Oracle2 - contract for an oracle of asset 2
    ///
    /// Factory - contract that mints the LP token for asset 1 & asset 2
    /// (SecretSwap - Pair | SiennaSwap - Exchange)
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct InitMsg {
        pub owner: String,
        pub oracle0: Contract,
        pub oracle1: Contract,
        pub factory: Contract,
        pub dex: Dex,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum Dex {
        SecretSwap,
        SiennaSwap,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        UpdateConfig {
            owner: Option<String>,
            oracle0: Option<Contract>,
            oracle1: Option<Contract>,
            factory: Option<Contract>,
            dex: Option<Dex>,
        },
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleAnswer {
        UpdateConfig { status: ResponseStatus },
    }

    // We define a custom struct for each query response
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct ConfigResponse {
        pub owner: String,
        pub oracle1: Contract,
        pub oracle2: Contract,
        pub factory: Contract,
        pub dex: Dex,
    }

    pub struct FairLpPriceInfo {
        pub reserve: u128,
        pub price: u128,
        pub decimals: u8,
    }

    /// Calculates the price of an LP token based on https://blog.alphafinance.io/fair-lp-token-pricing/.
    ///
    /// Assumes token prices are normalized to 10^18.
    pub fn get_fair_lp_token_price(
        a: FairLpPriceInfo,
        b: FairLpPriceInfo,
        total_supply: u128,
        lp_token_decimals: u8,
    ) -> StdResult<u128> {
        let normalized_reserve1 = Uint256::from(a.reserve * 10u128.pow((18 - a.decimals).into()));
        let normalized_reserve2 = Uint256::from(b.reserve * 10u128.pow((18 - b.decimals).into()));
        let normalized_supply =
            Uint256::from(total_supply * 10u128.pow((18 - lp_token_decimals).into()));
        let r = normalized_reserve1
            .checked_mul(normalized_reserve2)?
            .sqrt()?;
        let safe_price_a = Uint256::from(a.price);
        let safe_price_b = Uint256::from(b.price);
        let p = safe_price_a.checked_mul(safe_price_b)?.sqrt()?;
        let x = r
            .checked_mul(p)?
            .checked_div(normalized_supply)?
            .checked_mul(Uint256::from(2))?;
        x.clamp_u128()
    }
}

pub mod router {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use shared_types::{
        asset::{Contract, ResponseStatus},
        HandlePaddable,
    };

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "snake_case")]
    pub struct InitMsg {
        pub owner: String,
    }

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    #[serde(deny_unknown_fields)]
    pub enum RegistryOperation {
        Remove { key: String },
        Replace { oracle: Contract, key: String },
        Add { oracle: Contract, key: String },
    }

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    #[serde(deny_unknown_fields)]
    pub enum HandleMsg {
        ChangeOwner { new_owner: String },
        UpdateRegistry { operation: RegistryOperation },
        BatchUpdateRegistry { operations: Vec<RegistryOperation> },
    }

    impl HandlePaddable for HandleMsg {}

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    #[serde(deny_unknown_fields)]
    pub enum HandleAnswer {
        ChangeOwner { status: ResponseStatus },
        UpdateRegistry { status: ResponseStatus },
        BatchUpdateRegistry { status: ResponseStatus },
    }

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    #[serde(deny_unknown_fields)]
    pub enum QueryMsg {
        GetOwner {},
        /// Get oracle at that key
        GetOracle {
            key: String,
        },
        /// Get price of oracle at that key
        GetPrice {
            key: String,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    pub struct ConfigResponse {
        pub owner: String,
    }

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    pub struct OracleResponse {
        pub oracle: Contract,
    }

    pub mod querier {
        use crate::oracles::common::PriceResponse;
        use shared_types::scrt::{
            to_binary, HumanAddr, Querier, QueryRequest, StdResult, WasmQuery,
        };

        use super::QueryMsg;
        use super::*;

        pub fn query_price(
            contract: &Contract,
            querier: &impl Querier,
            key: String,
        ) -> StdResult<PriceResponse> {
            querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: HumanAddr(contract.address.clone()),
                callback_code_hash: contract.code_hash.clone(),
                msg: to_binary(&QueryMsg::GetPrice { key })?,
            }))
        }
    }
}
