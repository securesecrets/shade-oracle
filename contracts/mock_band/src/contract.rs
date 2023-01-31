use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, StdError, Storage, Uint128};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use shade_oracles::core::{pad_query_result, ResponseStatus};
use shade_oracles::interfaces::common::OraclePrice;
use shade_oracles::interfaces::providers::{
    mock::{
        Config,
        ExecuteAnswer,
        ExecuteMsg,
        InstantiateMsg,
        MockPrice,
        //ReferenceData,
    },
    BandQueryMsg, ReferenceData,
};
use shade_oracles::ssp::{Item, Map};
use shade_oracles::BLOCK_SIZE;

const MOCK_DATA: Map<(String, String), BandReferenceData> = Map::new("price-data");
const CONFIG: Item<Config> = Item::new("config");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let now = env.block.time.seconds();
    let admin_auth = msg.admin_auth.into_valid(deps.api)?;
    let quote_symbol = msg.quote_symbol.unwrap_or("USD".to_string());

    let config = Config {
        admin_auth,
        quote_symbol,
        enabled: true,
    };
    CONFIG.save(deps.storage, &config)?;

    for (base, quote, rate) in msg.initial_prices {
        MOCK_DATA.save(
            deps.storage,
            (base, quote),
            &BandReferenceData {
                rate,
                last_updated_base: now,
                last_updated_quote: now,
            },
        )?;
    }
    Ok(Response::default())
}

#[cw_serde]
pub struct BandReferenceData {
    pub rate: Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

fn require_enabled(config: &Config) -> StdResult<()> {
    if !config.enabled {
        return Err(StdError::generic_err("Contract is disabled"));
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    match msg {
        ExecuteMsg::SetStatus(status) => {
            config.require_admin(&deps.querier, info.sender)?;
            config.enabled = status;
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::default().add_attribute("action", "set_status"))
        }
        ExecuteMsg::SetPrice(price) => {
            require_enabled(&config)?;
            config.require_admin_or_bot(&deps.querier, info.sender)?;
            set_mock_price(deps.storage, env.block.time.seconds(), price)?;
            let data = to_binary(&ExecuteAnswer::SetPrice {
                status: ResponseStatus::Success,
            })?;

            Ok(Response::default()
                .set_data(data)
                .add_attribute("action", "set_price"))
        }
        ExecuteMsg::SetPrices(prices) => {
            require_enabled(&config)?;
            config.require_admin_or_bot(&deps.querier, info.sender)?;
            for price in prices {
                set_mock_price(deps.storage, env.block.time.seconds(), price)?;
            }
            let data = to_binary(&ExecuteAnswer::SetPrices {
                status: ResponseStatus::Success,
            })?;

            Ok(Response::default()
                .set_data(data)
                .add_attribute("action", "set_prices"))
        }
        ExecuteMsg::UpdateConfig {
            admin_auth,
            quote_symbol,
        } => {
            require_enabled(&config)?;
            config.require_admin(&deps.querier, info.sender)?;
            if let Some(admin_auth) = admin_auth {
                config.admin_auth = admin_auth.into_valid(deps.api)?;
            }
            if let Some(quote_symbol) = quote_symbol {
                config.quote_symbol = quote_symbol;
            }
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::default().add_attribute("action", "update_config"))
        }
    }
}

pub fn set_mock_price(storage: &mut dyn Storage, now: u64, price: MockPrice) -> StdResult<()> {
    MOCK_DATA.save(
        storage,
        (price.base_symbol, price.quote_symbol),
        &BandReferenceData {
            rate: price.rate,
            last_updated_base: price.last_updated.unwrap_or(now),
            last_updated_quote: price.last_updated.unwrap_or(now),
        },
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: BandQueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    pad_query_result(
        match msg {
            BandQueryMsg::GetReferenceData {
                base_symbol,
                quote_symbol,
            } => {
                require_enabled(&config)?;
                query_saved_band_data(deps, base_symbol, quote_symbol)
            }
            BandQueryMsg::GetReferenceDataBulk {
                base_symbols,
                quote_symbols,
            } => {
                require_enabled(&config)?;
                bulk_query_saved_band_data(deps, base_symbols, quote_symbols)
            }
            BandQueryMsg::GetPrice { key } => {
                require_enabled(&config)?;
                let data = MOCK_DATA.load(deps.storage, (key.clone(), config.quote_symbol))?;
                to_binary(&OraclePrice::new(
                    key,
                    ReferenceData {
                        rate: data.rate.into(),
                        last_updated_base: data.last_updated_base,
                        last_updated_quote: data.last_updated_quote,
                    },
                ))
            }
            BandQueryMsg::GetPrices { keys } => {
                require_enabled(&config)?;
                let mut results = vec![];
                for key in keys {
                    let data =
                        MOCK_DATA.load(deps.storage, (key.clone(), config.quote_symbol.clone()))?;
                    results.push(OraclePrice::new(
                        key,
                        ReferenceData {
                            rate: data.rate.into(),
                            last_updated_base: data.last_updated_base,
                            last_updated_quote: data.last_updated_quote,
                        },
                    ));
                }
                to_binary(&results)
            }
            BandQueryMsg::GetConfig {} => to_binary(&config),
        },
        BLOCK_SIZE,
    )
}

fn query_saved_band_data(
    deps: Deps,
    base_symbol: String,
    quote_symbol: String,
) -> StdResult<Binary> {
    to_binary(&MOCK_DATA.load(deps.storage, (base_symbol, quote_symbol))?)
}

fn bulk_query_saved_band_data(
    deps: Deps,
    base_symbols: Vec<String>,
    quote_symbols: Vec<String>,
) -> StdResult<Binary> {
    let mut results = vec![];

    for (base, quote) in base_symbols.iter().zip(quote_symbols) {
        results.push(MOCK_DATA.load(deps.storage, (base.to_string(), quote.to_string()))?);
    }
    to_binary(&results)
}
