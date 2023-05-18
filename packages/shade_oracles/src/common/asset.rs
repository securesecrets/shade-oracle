//! Defines how we'll store Assets in our contracts.
//! We care most about the token decimals, the asset contract itself, and the symbol used
//! to query the price via our oracle system so we can query prices for them.
use crate::error::CommonOracleError;
use crate::interfaces::common::OraclePrice;
use crate::querier::query_price;
use better_secret_math::common::{bankers_round, checked_add, exp10, muldiv};
use better_secret_math::U256;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, CosmosMsg, QuerierWrapper, StdError, StdResult, Storage, Uint256, Uint128};
use secret_storage_plus::Map;
use snip20::{
    helpers::query_token_info,
    msg::ExecuteMsg as Snip20ExecuteMsg
};
use shade_toolkit::{Contract, InstantiateCallback, ExecuteCallback, RawContract};

#[derive(Eq)]
#[cw_serde]
pub struct Asset {
    pub contract: Contract,
    pub decimals: u8,
    pub quote_symbol: String,
}

#[cw_serde]
pub enum AssetError {
    InvalidSymbol(String),
}

impl ToString for AssetError {
    fn to_string(&self) -> String {
        match self {
            AssetError::InvalidSymbol(s) => format!("Failed to query a price for {s}. Cannot set quote symbol of asset to invalid symbol."),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<StdError> for AssetError {
    fn into(self) -> StdError {
        StdError::generic_err(self.to_string())
    }
}

/// Map of assets.
pub struct Assets<'a, 'b>(pub Map<'a, &'b Addr, Asset>);

impl<'a, 'b> Assets<'a, 'b> {
    pub const fn new(namespace: &'a str) -> Self {
        Assets(Map::new(namespace))
    }
    pub fn require_existing_asset(
        &self,
        storage: &dyn Storage,
        address: &Addr,
    ) -> StdResult<Asset> {
        let asset = self.0.may_load(storage, address)?;
        if let Some(asset) = asset {
            Ok(asset)
        } else {
            Err(CommonOracleError::AssetNotFound(address.clone()).into())
        }
    }
    pub fn may_set(&self, storage: &mut dyn Storage, asset: &Asset) -> StdResult<()> {
        if self.0.may_load(storage, &asset.contract.address)?.is_none() {
            self.0.save(storage, &asset.contract.address, asset)?;
        }
        Ok(())
    }
    /// You can only update an existing asset's quote symbol because we assume their contract address, code hash, and token decimals are immutable.
    pub fn update_existing_asset(
        &self,
        storage: &mut dyn Storage,
        querier: &QuerierWrapper,
        oracle: &Contract,
        asset: &Addr,
        symbol: &str,
    ) -> StdResult<()> {
        let mut existing_asset = self.require_existing_asset(storage, asset)?;
        existing_asset.update_quote_symbol(oracle, querier, symbol.to_string())?;
        self.0.save(storage, asset, &existing_asset)?;
        Ok(())
    }
}

impl Default for Asset {
    fn default() -> Self {
        Self {
            contract: Contract {
                address: Addr::unchecked(String::default()),
                code_hash: Default::default(),
            },
            decimals: Default::default(),
            quote_symbol: Default::default(),
        }
    }
}

impl Asset {
    pub fn new(contract: Contract, decimals: u8, quote_symbol: String) -> Self {
        Asset {
            contract,
            decimals,
            quote_symbol,
        }
    }
    pub fn update_quote_symbol(
        &mut self,
        oracle: &Contract,
        querier: &QuerierWrapper,
        symbol: String,
    ) -> StdResult<()> {
        let resp = query_price(oracle, querier, symbol.clone());
        if resp.is_err() {
            return Err(AssetError::InvalidSymbol(symbol).into());
        } else {
            self.quote_symbol = symbol;
        }
        Ok(())
    }
    pub fn get_price(&self, querier: &QuerierWrapper, router: &Contract) -> StdResult<OraclePrice> {
        query_price(router, querier, self.quote_symbol.clone())
    }
    /// Normalizes the asset amount from being based off asset decimals -> 18 decimals.
    pub fn normalize_amount(&self, amount: impl Into<U256>) -> StdResult<U256> {
        if self.decimals == 18 {
            Ok(amount.into())
        } else {
            muldiv(amount.into(), exp10(18), exp10(self.decimals))
        }
    }
    /// Gets the amount of asset the amount normalized to 18 decimals represents.
    pub fn get_amount(&self, normalized_amount: impl Into<U256>) -> StdResult<U256> {
        if self.decimals == 18 {
            Ok(normalized_amount.into())
        } else {
            let precision_diff = 18 - self.decimals;
            let amount =
                bankers_round(normalized_amount.into(), precision_diff) / exp10(precision_diff);
            Ok(amount)
        }
    }
    pub fn append_msgs(
        &self,
        msgs: &mut Vec<CosmosMsg>,
        new_msgs: Vec<Snip20ExecuteMsg>,
    ) -> StdResult<()> {
        let mut cosmos_msgs = vec![];
        for msg in new_msgs {
            cosmos_msgs.push(msg.to_cosmos_msg(&self.contract, vec![])?);
        }
        msgs.append(&mut cosmos_msgs);
        Ok(())
    }
}

#[derive(Default)]
#[cw_serde]
pub struct RawAsset {
    pub contract: RawContract,
    pub quote_symbol: String,
}

impl RawAsset {
    pub fn new(contract: impl Into<RawContract>, quote_symbol: impl Into<String>) -> Self {
        RawAsset {
            contract: contract.into(),
            quote_symbol: quote_symbol.into(),
        }
    }
    pub fn into_asset(
        self,
        oracle: &Contract,
        querier: &QuerierWrapper,
        api: &dyn Api,
    ) -> StdResult<Asset> {
        let resp = query_price(oracle, querier, self.quote_symbol.clone());
        if resp.is_err() {
            return Err(AssetError::InvalidSymbol(self.quote_symbol).into());
        }
        let contract = self.contract.clone().validate(api)?;
        let decimals = query_token_info(querier, &contract)?.decimals;
        Ok(Asset::new(contract, decimals, self.quote_symbol))
    }
    pub fn into_asset_without_symbol_check(
        self,
        api: &dyn Api,
        querier: &QuerierWrapper,
    ) -> StdResult<Asset> {
        let contract = self.contract.clone().validate(api)?;
        let decimals = query_token_info(querier, &contract)?.decimals;
        Ok(Asset::new(contract, decimals, self.quote_symbol))
    }
}

make_btr! {
    /// A struct containing an amount of some token and the value of that amount.
    #[derive(Default)]
    ValuedAmount {
        amount: Uint256, U256, "Token amount of valued asset (18 decimals).";
        value: Uint256, U256, "Value of token amount at time (18 decimals)."
    }
}

impl BtrValuedAmount {
    pub fn add(mut self, amount: U256, value: U256) -> StdResult<Self> {
        self.amount = checked_add(self.amount, amount)?;
        self.value = checked_add(self.value, value)?;
        Ok(self)
    }
    pub fn add_another(&mut self, another: &BtrValuedAmount) {
        self.amount += another.amount;
        self.value += another.value;
    }
    pub fn is_zero(&self) -> bool {
        self.amount.le(&U256::ZERO) && self.value.le(&U256::ZERO)
    }
}
