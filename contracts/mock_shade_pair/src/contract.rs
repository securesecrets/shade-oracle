use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, DepsMut, MessageInfo, Uint128};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Response, StdError, StdResult};
use shade_oracles::{
    core::{Contract, ExecuteCallback, InstantiateCallback},
    interfaces::dex::shadeswap::{
        Fee, FeeInfo, PairInfoResponse, ShadeSwapQueryMsg, SwapResult, SwapSimulationResponse,
        TokenPair, TokenType,
    },
    ssp::Item,
};

pub fn pool_take_amount(give_amount: Uint128, give_pool: Uint128, take_pool: Uint128) -> Uint128 {
    Uint128::from(
        take_pool.u128() - give_pool.u128() * take_pool.u128() / (give_pool + give_amount).u128(),
    )
}

#[cw_serde]
pub struct InstantiateMsg {}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

const PAIR_INFO: Item<PairInfoResponse> = Item::new("pair_info");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[cw_serde]
pub enum ExecuteMsg {
    MockPool {
        token_a: Contract,
        amount_a: Uint128,
        token_b: Contract,
        amount_b: Uint128,
    },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::MockPool {
            token_a,
            amount_a,
            token_b,
            amount_b,
        } => {
            PAIR_INFO.save(
                deps.storage,
                &PairInfoResponse {
                    liquidity_token: Contract {
                        address: Addr::unchecked("".to_string()),
                        code_hash: "".to_string(),
                    },
                    factory: Some(Contract {
                        address: Addr::unchecked("".to_string()),
                        code_hash: "".to_string(),
                    }),
                    pair: TokenPair(
                        TokenType::CustomToken {
                            contract_addr: token_a.address,
                            token_code_hash: token_a.code_hash,
                            oracle_key: None,
                        },
                        TokenType::CustomToken {
                            contract_addr: token_b.address,
                            token_code_hash: token_b.code_hash,
                            oracle_key: None,
                        },
                        false,
                    ),
                    amount_0: amount_a,
                    amount_1: amount_b,
                    total_liquidity: Uint128::zero(),
                    contract_version: 1,
                    fee_info: FeeInfo {
                        shade_dao_address: Addr::unchecked("".to_string()),
                        lp_fee: Fee::default(),
                        shade_dao_fee: Fee::default(),
                        stable_lp_fee: Fee::default(),
                        stable_shade_dao_fee: Fee::default(),
                    },
                },
            )?;

            Ok(Response::default())
        }
    }

    // TODO: actual swap execute
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: ShadeSwapQueryMsg) -> StdResult<Binary> {
    match msg {
        ShadeSwapQueryMsg::GetPairInfo {} => to_binary(&PAIR_INFO.load(deps.storage)?),
        ShadeSwapQueryMsg::SwapSimulation { offer, exclude_fee } => {
            //TODO: check swap doesnt exceed pool size

            let in_token = match offer.token {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                    oracle_key,
                } => Contract {
                    address: contract_addr,
                    code_hash: token_code_hash,
                },
                _ => {
                    return Err(StdError::generic_err("Only CustomToken supported"));
                }
            };

            let pair_info = PAIR_INFO.load(deps.storage)?;

            match pair_info.pair.0 {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash: _,
                    oracle_key: None,
                } => {
                    if in_token.address == contract_addr {
                        return to_binary(&SwapSimulationResponse {
                            price: String::default(),
                            total_fee_amount: Uint128::default(),
                            lp_fee_amount: Uint128::default(),
                            shade_dao_fee_amount: Uint128::default(),
                            result: SwapResult {
                                return_amount: pool_take_amount(
                                    offer.amount,
                                    pair_info.amount_0,
                                    pair_info.amount_1,
                                ),
                            },
                        });
                    }
                }
                _ => {
                    return Err(StdError::generic_err("Only CustomToken supported"));
                }
            };

            match pair_info.pair.1 {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash: _,
                    oracle_key: None,
                } => {
                    if in_token.address == contract_addr {
                        return to_binary(&SwapSimulationResponse {
                            price: String::default(),
                            total_fee_amount: Uint128::default(),
                            lp_fee_amount: Uint128::default(),
                            shade_dao_fee_amount: Uint128::default(),
                            result: SwapResult {
                                return_amount: pool_take_amount(
                                    offer.amount,
                                    pair_info.amount_1,
                                    pair_info.amount_0,
                                ),
                            },
                        });
                    }
                }
                _ => {
                    return Err(StdError::generic_err("Only CustomToken supported"));
                }
            };

            Err(StdError::generic_err("Failed to match offer token"))
        }
    }
}
