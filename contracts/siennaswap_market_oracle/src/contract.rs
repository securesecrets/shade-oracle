use cosmwasm_std::{
    entry_point, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError, StdResult,
    Uint128,
};
use shade_oracles::{
    core::Query,
    core::{snip20::helpers::TokenInfo, Contract},
    interfaces::common::{OraclePrice, OracleQuery},
    interfaces::{
        band::ReferenceData,
        CommonConfig,
    },
    interfaces::dex::lp_market::{
        msg::*, LiquidityPairMarketOracle,
    },
    math::TokenMath,
    protocols::siennaswap::{
        SiennaSwapQuerier
    },
    querier::{query_band_price, query_price, query_token_info},
    ssp::ItemStorage,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = SiennaswapMarketOracle.init_config(deps.storage, deps.api, msg.config)?;
    let pair = msg.pair.into_valid(deps.api)?;
    let pair_info_response = SiennaSwapQuerier::query_pair_info(&deps.querier, &pair)?;
    let tokens: [Contract; 2] = 

    let token_infos: [TokenInfo; 2] = tokens
        .iter()
        .map(|t| query_token_info(t, &deps.querier).ok().unwrap())
        .collect::<Vec<TokenInfo>>()
        .try_into()
        .ok()
        .unwrap();

    let primary_i = match token_infos.iter().position(|t| t.symbol == msg.symbol) {
        Some(i) => i,
        None => {
            return Err(StdError::generic_err(format!(
                "Neither token matches {}",
                msg.symbol
            )));
        }
    };

    let base_i = token_infos
        .iter()
        .position(|t| t.symbol != msg.symbol)
        .unwrap();

    let market_data = MarketData {
        base_peg: match msg.base_peg {
            Some(p) => p,
            None => token_infos[base_i].symbol.clone(),
        },
        symbol: msg.symbol.clone(),
        pair,
    };

    if let Err(e) = query_price(&config.router, &deps.querier, market_data.base_peg.clone()) {
        return Err(StdError::generic_err(format!(
            "Failed to query base_peg {} from router {}; {}",
            market_data.base_peg, config.router.address, e
        )));
    };

    market_data.save(deps.storage)?;
    PRIMARY_TOKEN.save(deps.storage, &tokens[primary_i].clone())?;
    PRIMARY_INFO.save(deps.storage, &token_infos[primary_i])?;
    BASE_INFO.save(deps.storage, &token_infos[base_i])?;

    Ok(Response::default())
}

pub fn fetch_target_token_info(
    &self,
    querier: &QuerierWrapper,
    target_token: &Addr,
    tokens: [Contract; 2],
) -> StdResult<TokenInfo> {
    // let pair = pair.into_valid(api)?;
    // let target_token = api.addr_validate(&target_token)?;
    // let base_token = base_token.into_asset(&self.config.router, querier, api)?;

    // let tokens = DexQuerier::fetch_shadeswap_pair(
    //     querier,
    //     &pair,
    //     &base_token.contract.address,
    //     &target_token,
    // )?;

    // Check which token in the pair is the target asset. One is guaranteed to be the target asset because of the check above.
    let target_token = if tokens[0].address.eq(target_token) {
        &tokens[0]
    } else {
        &tokens[1]
    };

    query_token_info(target_token, querier)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
    ExecuteMsg::UpdateConfig { updates } => todo!(),
}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: OracleQuery) -> StdResult<QueryResponse> {
}

    pub fn try_query_price(
        &self,
        deps: Deps,
        _env: &Env,
        key: String,
        config: &CommonConfig,
    ) -> StdResult<OraclePrice> {
        let primary_token: Contract = PRIMARY_TOKEN.load(deps.storage)?;
        let primary_info = PRIMARY_INFO.load(deps.storage)?;
        let market_config = MarketData::load(deps.storage)?;

        // Simulate trade 1 primary -> 1 base
        let sim: SimulationResponse = SiennaSwapExchangeQueryMsg::SwapSimulation {
            offer: TokenTypeAmount {
                amount: Uint128::from(10u128.pow(primary_info.decimals.into())),
                token: SiennaDexTokenType::CustomToken {
                    contract_addr: primary_token.address,
                    token_code_hash: primary_token.code_hash,
                },
            },
        }
        .query(&deps.querier, &market_config.pair)?;

        // Normalize to 'rate * 10^18'
        let base_info = BASE_INFO.load(deps.storage)?;
        let exchange_rate: Uint128 =
            TokenMath::normalize_value(sim.return_amount, base_info.decimals)?.into();

        // Query router for base_peg/USD
        let base_usd_price = if config.only_band {
            query_band_price(&config.router, &deps.querier, market_config.base_peg)?
        } else {
            query_price(&config.router, &deps.querier, market_config.base_peg)?
        };

        // Translate price to primary/USD
        let price = base_usd_price
            .data()
            .rate
            .multiply_ratio(exchange_rate, 10u128.pow(18));

        Ok(OraclePrice::new(
            key,
            ReferenceData {
                rate: price,
                last_updated_base: base_usd_price.data().last_updated_base,
                last_updated_quote: base_usd_price.data().last_updated_quote,
            },
        ))
    }
