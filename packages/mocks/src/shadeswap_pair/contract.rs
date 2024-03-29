use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, DepsMut, MessageInfo, Uint128};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Response, StdError, StdResult};
use shade_oracles::protocols::shadeswap::{PairInfo, ShadeSwapQueryMsgResponse};
use shade_oracles::{
    core::{Contract, ExecuteCallback, InstantiateCallback},
    protocols::shadeswap::{
        Fee, FeeInfo, QueryMsg, SwapResult, SwapSimulationResponse, TokenPair, TokenType,
    },
    ssp::Item,
};
use shade_protocol::contract_interfaces::snip20::ExecuteMsg as Snip20ExecuteMsg;

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

const PAIR_INFO: Item<PairInfo> = Item::new("pair_info");

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
        liquidity_token: Contract,
        liquidity_tokens: Uint128,
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
            liquidity_token,
            liquidity_tokens,
        } => {
            PAIR_INFO.save(
                deps.storage,
                &PairInfo {
                    liquidity_token: liquidity_token.clone(),
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
                    total_liquidity: liquidity_tokens,
                    contract_version: 1,
                    fee_info: FeeInfo {
                        shade_dao_address: Addr::unchecked("".to_string()),
                        lp_fee: Fee::default(),
                        shade_dao_fee: Fee::default(),
                        stable_lp_fee: Fee::default(),
                        stable_shade_dao_fee: Fee::default(),
                    },
                    stable_info: None,
                },
            )?;

            let mut msgs = vec![];
            msgs.push(Snip20ExecuteMsg::Mint {
                recipient: Addr::unchecked("voidvoidvoid").to_string(),
                amount: amount_a,
                memo: None,
                padding: None,
            }.to_cosmos_msg(&liquidity_token, vec![])?);

            Ok(Response::default().add_messages(msgs))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPairInfo {} => {
            let PairInfo {
                liquidity_token,
                factory,
                pair,
                amount_0,
                amount_1,
                total_liquidity,
                contract_version,
                fee_info,
                stable_info,
            } = PAIR_INFO.load(deps.storage)?;
            to_binary(&ShadeSwapQueryMsgResponse::GetPairInfo {
                liquidity_token,
                factory,
                pair,
                amount_0,
                amount_1,
                total_liquidity,
                contract_version,
                fee_info,
                stable_info,
            })
        }
        QueryMsg::SwapSimulation {
            offer,
            exclude_fee: _,
        } => {
            let in_token = match offer.token {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                    oracle_key: _,
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
                        return to_binary(&ShadeSwapQueryMsgResponse::SwapSimulation {
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
