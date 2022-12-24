//! Defines how we'll store Assets in our contracts.
//! We care most about the token decimals, the asset contract itself, and the symbol used
//! to query the price via our oracle system so we can query prices for them.
use crate::error::CommonOracleError;
use crate::interfaces::common::{BtrOraclePrice, OraclePrice};
use crate::querier::query_price;
use better_secret_math::core::checked_add;
use better_secret_math::{BtrRebase, U256};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, CosmosMsg, QuerierWrapper, StdError, StdResult, Storage, Uint256};
use secret_storage_plus::Map;
use shade_protocol::{
    contract_interfaces::snip20::helpers::token_info,
    contract_interfaces::snip20::ExecuteMsg as Snip20ExecuteMsg,
};
use shade_protocol::{
    utils::{asset::RawContract, ExecuteCallback},
    Contract,
};

#[derive(Eq)]
#[cw_serde]
pub struct Asset {
    pub contract: Contract,
    pub decimals: u8,
    pub quote_symbol: String,
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
        if asset.is_none() {
            Err(CommonOracleError::AssetNotFound(address.clone()).into())
        } else {
            Ok(asset.unwrap())
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
        symbol: &String,
    ) -> StdResult<()> {
        self.0
            .update(storage, &asset, |old_asset| -> StdResult<_> {
                match old_asset {
                    Some(mut asset) => {
                        asset.update_quote_symbol(&oracle, &querier, symbol.clone())?;
                        Ok(asset)
                    }
                    None => Err(StdError::generic_err(format!(
                        "{} is not an existing asset.",
                        asset.clone()
                    ))),
                }
            })?;
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
            return Err(StdError::generic_err(format!(
                "Failed to query a price for {}. Cannot set quote symbol of asset to faulty symbol.",
                symbol
            )));
        } else {
            self.quote_symbol = symbol;
        }
        Ok(())
    }
    pub fn get_price(&self, querier: &QuerierWrapper, router: &Contract) -> StdResult<OraclePrice> {
        query_price(router, querier, self.quote_symbol.clone())
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
            return Err(StdError::generic_err(format!(
                "Failed to query a price for {}. Cannot set quote symbol of asset to invalid symbol.",
                self.quote_symbol
            )));
        }
        let contract = self.contract.clone().into_valid(api)?;
        let decimals = token_info(querier, &contract)?.decimals;
        Ok(Asset::new(contract, decimals, self.quote_symbol))
    }
    pub fn into_asset_without_symbol_check(
        self,
        api: &dyn Api,
        querier: &QuerierWrapper,
    ) -> StdResult<Asset> {
        let contract = self.contract.clone().into_valid(api)?;
        let decimals = token_info(querier, &contract)?.decimals;
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

make_btr! {
    /// The valued amount of some tokens associated with a rebase.
    #[derive(Default)]
    ValuedRebaseAmount {
        base: ValuedAmount, BtrValuedAmount, "";
        elastic: ValuedAmount, BtrValuedAmount, ""
    }
}

impl BtrValuedRebaseAmount {
    pub fn init(
        base_amount: U256,
        elastic_amount: U256,
        base_value: U256,
        elastic_value: U256,
    ) -> Self {
        BtrValuedRebaseAmount {
            base: BtrValuedAmount::new(base_amount, base_value),
            elastic: BtrValuedAmount::new(elastic_amount, elastic_value),
        }
    }
    pub fn from_base(rebase: &BtrRebase, base: U256, price: &BtrOraclePrice) -> StdResult<Self> {
        let elastic = rebase.to_elastic(base, false)?;
        let elastic_value = price.calc_value(elastic)?;
        let base_value = price.calc_value(base)?;
        Ok(Self::init(base, elastic, base_value, elastic_value))
    }
    pub fn from_elastic(
        rebase: &BtrRebase,
        elastic: U256,
        price: &BtrOraclePrice,
    ) -> StdResult<Self> {
        let base = rebase.to_base(elastic, false)?;
        let base_value = price.calc_value(base)?;
        let elastic_value = price.calc_value(elastic)?;
        Ok(Self::init(base, elastic, base_value, elastic_value))
    }
    pub fn from_safe(amount: U256, price: &BtrOraclePrice) -> StdResult<Self> {
        let value = price.calc_value(amount)?;
        Ok(Self::init(amount, value, amount, value))
    }
}
