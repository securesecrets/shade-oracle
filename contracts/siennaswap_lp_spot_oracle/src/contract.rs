use cosmwasm_std::{entry_point, Uint128};
use cosmwasm_std::{
    to_binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, QueryResponse, Response, StdResult,
    WasmQuery,
};
use shade_oracles::common::{oracle_exec, CommonConfig, Oracle};
use shade_oracles::{
    common::querier::{query_prices, query_token_info},
    common::{ExecuteMsg, OraclePrice, OracleQuery},
    core::pad_query_result,
    interfaces::band::ReferenceData,
    interfaces::lp::{
        math::{get_lp_token_spot_price, FairLpPriceInfo},
        siennaswap::{resolve_pair, ConfigResponse, InstantiateMsg, PairData, EXCHANGE},
    },
    protocols::siennaswap::{SiennaSwapExchangeQueryMsg, SiennaSwapPairInfoResponse},
    ssp::ItemStorage,
    BLOCK_SIZE,
};
use std::cmp::min;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let pair_info_response: SiennaSwapPairInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.exchange.address.to_string(),
            code_hash: msg.exchange.code_hash.clone(),
            msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
        }))?;
    let pair_info = pair_info_response.pair_info;
    let tokens = resolve_pair(&pair_info)?;
    let token0_decimals = query_token_info(&tokens.0, &deps.querier)?.decimals;
    let token1_decimals = query_token_info(&tokens.1, &deps.querier)?.decimals;
    let lp_token = pair_info.liquidity_token;

    let pair = PairData {
        lp_token,
        token0_decimals,
        token1_decimals,
        symbol_0: msg.symbol_0,
        symbol_1: msg.symbol_1,
    };

    pair.save(deps.storage)?;
    SiennaswapLpOracle.init_config(deps.storage, deps.api, msg.config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    oracle_exec(deps, env, info, msg, SiennaswapLpOracle)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: OracleQuery) -> StdResult<QueryResponse> {
    let config = CommonConfig::load(deps.storage)?;

    pad_query_result(
        match msg {
            OracleQuery::GetConfig {} => to_binary(&ConfigResponse {
                config,
                exchange: EXCHANGE.load(deps.storage)?,
                pair: PairData::load(deps.storage)?,
            }),
            OracleQuery::GetPrice { key } => {
                SiennaswapLpOracle.can_query_price(deps, &key)?;
                to_binary(
                    &SiennaswapLpOracle
                        .price_resp(SiennaswapLpOracle.try_query_price(deps, &_env, key, &config)?),
                )
            }
            OracleQuery::GetPrices { keys } => {
                SiennaswapLpOracle.can_query_prices(deps, &keys)?;
                to_binary(
                    &SiennaswapLpOracle.prices_resp(
                        SiennaswapLpOracle.try_query_prices(deps, &_env, keys, &config)?,
                    ),
                )
            }
        },
        BLOCK_SIZE,
    )
}

pub struct SiennaswapLpOracle;

impl Oracle for SiennaswapLpOracle {
    fn try_query_price(
        &self,
        deps: Deps,
        _env: &Env,
        key: String,
        config: &shade_oracles::common::CommonConfig,
    ) -> StdResult<OraclePrice> {
        let pair = PairData::load(deps.storage)?;
        let exchange = EXCHANGE.load(deps.storage)?;

        let prices = query_prices(
            &config.router,
            &deps.querier,
            [pair.symbol_0.clone(), pair.symbol_1.clone()].as_slice(),
        )?;

        let mut price0 = OraclePrice::default();
        let mut price1 = OraclePrice::default();
        for price in prices {
            if price.key.eq(&pair.symbol_0) {
                price0 = price;
            } else {
                price1 = price;
            }
        }

        let pair_info_response: SiennaSwapPairInfoResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: exchange.address.to_string(),
                code_hash: exchange.code_hash,
                msg: to_binary(&SiennaSwapExchangeQueryMsg::PairInfo)?,
            }))?;
        let pair_info = pair_info_response.pair_info;
        let reserve0 = pair_info.amount_0;
        let reserve1 = pair_info.amount_1;

        let lp_token_info = query_token_info(&pair.lp_token, &deps.querier)?;

        let total_supply = lp_token_info.total_supply.unwrap();
        let lp_token_decimals = lp_token_info.decimals;

        let a = FairLpPriceInfo {
            reserve: reserve0.u128(),
            price: price0.data().rate.u128(),
            decimals: pair.token0_decimals,
        };

        let b = FairLpPriceInfo {
            reserve: reserve1.u128(),
            price: price1.data().rate.u128(),
            decimals: pair.token1_decimals,
        };

        let price = get_lp_token_spot_price(a, b, total_supply.u128(), lp_token_decimals);

        let data = ReferenceData {
            rate: Uint128::from(u128::from_be_bytes(price.unwrap().to_be_bytes())),
            last_updated_base: min(
                price0.data().last_updated_base,
                price1.data().last_updated_base,
            ),
            last_updated_quote: min(
                price0.data().last_updated_quote,
                price1.data().last_updated_quote,
            ),
        };
        Ok(OraclePrice::new(key, data))
    }
}
