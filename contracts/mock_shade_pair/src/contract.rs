use cosmwasm_std::{
    to_binary,
    Api,
    Binary,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    InitResponse,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use secret_toolkit::utils::InitCallback;
use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::Contract,
    storage::Item,
    protocols::shadeswap::{
        TokenType,
        TokenPair,
        ShadeSwapQueryMsg,
        PairInfoResponse,
        EstimatedPriceResponse,
    },
};
use fadroma::scrt::ContractLink;

pub fn pool_take_amount(
    give_amount: Uint128,
    give_pool: Uint128,
    take_pool: Uint128,
) -> Uint128 {
    Uint128(
        take_pool.u128() - give_pool.u128() * take_pool.u128() / (give_pool + give_amount).u128(),
    )
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

const PAIR_INFO: Item<PairInfoResponse> = Item::new("pair_info");

pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    Ok(InitResponse::default())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    MockPool {
        token_a: Contract,
        amount_a: Uint128,
        token_b: Contract,
        amount_b: Uint128,
    },
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::MockPool {
            token_a,
            amount_a,
            token_b,
            amount_b,
        } => {
            PAIR_INFO.save(&mut deps.storage, &PairInfoResponse {
                liquidity_token: ContractLink {
                    address: HumanAddr("".to_string()),
                    code_hash: "".to_string(),
                },
                factory: ContractLink {
                    address: HumanAddr("".to_string()),
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
            })?;

            Ok(HandleResponse::default())
        }
    }

    // TODO: actual swap handle
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: ShadeSwapQueryMsg,
) -> StdResult<Binary> {
    match msg {
        ShadeSwapQueryMsg::GetPairInfo => to_binary(&PAIR_INFO.load(&deps.storage)?),
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

            let pair_info = PAIR_INFO.load(&deps.storage)?;

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

            return Err(StdError::generic_err("Failed to match offer token"));
        }
    }
}
