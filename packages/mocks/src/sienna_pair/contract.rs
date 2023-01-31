use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, DepsMut, MessageInfo, Uint128};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Response, StdError, StdResult, Storage};
use shade_oracles::storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use shade_oracles::{
    core::{Contract, ExecuteCallback, InstantiateCallback},
    protocols::siennaswap::{
        Pair, PairInfo, PairInfoResponse, QueryMsg, SimulationResponse, TokenType,
    },
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

pub static PAIR_INFO: &[u8] = b"pair_info";

pub fn pair_info_r(storage: &dyn Storage) -> ReadonlySingleton<PairInfo> {
    singleton_read(storage, PAIR_INFO)
}

pub fn pair_info_w(storage: &mut dyn Storage) -> Singleton<PairInfo> {
    singleton(storage, PAIR_INFO)
}

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
            let pair_info = PairInfo {
                liquidity_token: Contract {
                    address: Addr::unchecked("".to_string()),
                    code_hash: "".to_string(),
                },
                factory: Contract {
                    address: Addr::unchecked("".to_string()),
                    code_hash: "".to_string(),
                },
                pair: Pair {
                    token_0: TokenType::CustomToken {
                        contract_addr: token_a.address,
                        token_code_hash: token_a.code_hash,
                    },
                    token_1: TokenType::CustomToken {
                        contract_addr: token_b.address,
                        token_code_hash: token_b.code_hash,
                    },
                },
                amount_0: amount_a,
                amount_1: amount_b,
                total_liquidity: Uint128::zero(),
                contract_version: 0,
            };

            pair_info_w(deps.storage).save(&pair_info)?;

            Ok(Response::default())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PairInfo => to_binary(&PairInfoResponse {
            pair_info: pair_info_r(deps.storage).load()?,
        }),
        QueryMsg::SwapSimulation { offer } => {
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

            let pair_info = pair_info_r(deps.storage).load()?;

            match pair_info.pair.token_0 {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash: _,
                } => {
                    if in_token.address == contract_addr {
                        return to_binary(&SimulationResponse {
                            return_amount: pool_take_amount(
                                offer.amount,
                                pair_info.amount_0,
                                pair_info.amount_1,
                            ),
                            spread_amount: Uint128::zero(),
                            commission_amount: Uint128::zero(),
                        });
                    }
                }
                _ => {
                    return Err(StdError::generic_err("Only CustomToken supported"));
                }
            };

            match pair_info.pair.token_1 {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash: _,
                } => {
                    if in_token.address == contract_addr {
                        return to_binary(&SimulationResponse {
                            return_amount: pool_take_amount(
                                offer.amount,
                                pair_info.amount_1,
                                pair_info.amount_0,
                            ),
                            spread_amount: Uint128::zero(),
                            commission_amount: Uint128::zero(),
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
