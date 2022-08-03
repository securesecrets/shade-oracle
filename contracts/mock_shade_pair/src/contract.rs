use cosmwasm_std::{entry_point, DepsMut, MessageInfo, Uint128};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Response, StdError, StdResult};
use shade_oracles::core::cosmwasm_schema::cw_serde;
use shade_oracles::{
    core::{Contract, ExecuteCallback, InstantiateCallback},
    protocols::shadeswap::{
        EstimatedPriceResponse, PairInfoResponse, ShadeSwapQueryMsg, TokenPair, TokenType,
    },
    storage::Item,
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

#[entry_point]
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

#[entry_point]
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
                    factory: Contract {
                        address: Addr::unchecked("".to_string()),
                        code_hash: "".to_string(),
                    },
                    pair: TokenPair(
                        TokenType::CustomToken {
                            contract_addr: token_a.address,
                            token_code_hash: token_a.code_hash,
                        },
                        TokenType::CustomToken {
                            contract_addr: token_b.address,
                            token_code_hash: token_b.code_hash,
                        },
                    ),
                    amount_0: amount_a,
                    amount_1: amount_b,
                    total_liquidity: Uint128::zero(),
                    contract_version: 0,
                },
            )?;

            Ok(Response::default())
        }
    }

    // TODO: actual swap execute
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: ShadeSwapQueryMsg) -> StdResult<Binary> {
    match msg {
        ShadeSwapQueryMsg::GetPairInfo => to_binary(&PAIR_INFO.load(deps.storage)?),
        ShadeSwapQueryMsg::GetEstimatedPrice { offer } => {
            //TODO: check swap doesnt exceed pool size

            let in_token = match offer.token {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
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
                } => {
                    if in_token.address == contract_addr {
                        return to_binary(&EstimatedPriceResponse {
                            estimated_price: pool_take_amount(
                                offer.amount,
                                pair_info.amount_0,
                                pair_info.amount_1,
                            ),
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
                } => {
                    if in_token.address == contract_addr {
                        return to_binary(&EstimatedPriceResponse {
                            estimated_price: pool_take_amount(
                                offer.amount,
                                pair_info.amount_1,
                                pair_info.amount_0,
                            ),
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
