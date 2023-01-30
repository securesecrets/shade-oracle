use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, Addr, StdError, Uint128};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use secret_storage_plus::{Item, Map};
use shade_protocol::snip20::QueryAnswer;
use shade_protocol::utils::{ExecuteCallback, Query};
use shade_protocol::{
    contract_interfaces::snip20::{
        helpers::token_info, ExecuteMsg as Snip20ExecuteMsg, QueryMsg as Snip20QueryMsg,
    },
    utils::{asset::RawContract, pad_handle_result, pad_query_result},
    Contract, BLOCK_SIZE,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
    pub tokens: Vec<RawContract>,
    pub mint_frequency: u64,
    pub mint_amount: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
    Toggle {},
    UpdateConfig {
        admins: Option<Vec<String>>,
        mint_frequency: Option<u64>,
        mint_amount: Option<Uint128>,
    },
    AddTokens(Vec<RawContract>),
}

#[cw_serde]
pub enum QueryMsg {
    GetConfig {},
    GetMintStatus(String),
}

#[cw_serde]
pub struct Config {
    pub enabled: bool,
    pub admins: Vec<Addr>,
    pub tokens: Vec<TokenData>,
    pub mint_frequency: u64,
    pub mint_amount: Uint128,
}

impl Config {
    pub fn require_admin(&self, addr: &Addr) -> StdResult<()> {
        if !self.admins.contains(addr) {
            return Err(StdError::generic_err(format!(
                "Address {addr} is not an admin",
            )));
        }
        Ok(())
    }
    pub fn require_enabled(&self) -> StdResult<()> {
        if !self.enabled {
            return Err(StdError::generic_err("Faucet is disabled."));
        }
        Ok(())
    }
    pub fn require_can_mint(&self, now: u64, data: &UserData) -> StdResult<()> {
        let time_since_last_mint = now - data.last_minted;
        if time_since_last_mint < self.mint_frequency {
            return Err(StdError::generic_err(format!(
                "Faucet is on cooldown. Try again in {} seconds.",
                self.mint_frequency - time_since_last_mint
            )));
        }
        Ok(())
    }
}

#[cw_serde]
#[derive(Default)]
pub struct UserData {
    pub last_minted: u64,
}

#[cw_serde]
pub struct TokenData {
    pub contract: Contract,
    pub decimals: u8,
}

impl TokenData {
    pub fn into_valid(faucet: &Addr, deps: Deps, raw: RawContract) -> StdResult<Self> {
        let contract = raw.into_valid(deps.api)?;
        let token_info = token_info(&deps.querier, &contract)?;
        let resp: QueryAnswer = Snip20QueryMsg::Minters {}.query(&deps.querier, &contract)?;
        let minters = match resp {
            QueryAnswer::Minters { minters } => minters,
            _ => return Err(StdError::generic_err("Invalid response from token.")),
        };
        if !minters.contains(faucet) {
            return Err(StdError::generic_err(format!(
                "Faucet {} is not a minter for {}.",
                faucet, contract.address,
            )));
        }
        Ok(Self {
            contract,
            decimals: token_info.decimals,
        })
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const USER_DATA: Map<&Addr, UserData> = Map::new("user_data");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let admins = if msg.admins.is_empty() {
        vec![info.sender]
    } else {
        let admins: Result<Vec<_>, _> = msg
            .admins
            .into_iter()
            .map(|a| deps.api.addr_validate(&a))
            .collect();
        admins?
    };

    let tokens: Result<Vec<_>, _> = msg
        .tokens
        .into_iter()
        .map(|t| TokenData::into_valid(&env.contract.address, deps.as_ref(), t))
        .collect();

    let config = Config {
        enabled: true,
        admins,
        tokens: tokens?,
        mint_frequency: msg.mint_frequency,
        mint_amount: msg.mint_amount,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut resp = Response::default();
    let resp = match msg {
        ExecuteMsg::Mint {} => {
            config.require_enabled()?;
            let mut user_data = USER_DATA
                .may_load(deps.storage, &info.sender)?
                .unwrap_or_default();
            let now = env.block.time.seconds();
            config.require_can_mint(now, &user_data)?;
            let mut msgs = vec![];
            for token in config.tokens.iter() {
                let mint_amount =
                    config.mint_amount * Uint128::new(10u128.pow(token.decimals as u32));
                msgs.push(
                    Snip20ExecuteMsg::Mint {
                        recipient: info.sender.to_string(),
                        amount: mint_amount,
                        memo: None,
                        padding: None,
                    }
                    .to_cosmos_msg(&token.contract, vec![])?,
                )
            }
            user_data.last_minted = now;
            USER_DATA.save(deps.storage, &info.sender, &user_data)?;
            Ok(resp
                .add_messages(msgs)
                .add_attribute("action", "mint_tokens"))
        }
        _ => {
            config.require_admin(&info.sender)?;
            match msg {
                ExecuteMsg::Toggle {} => {
                    config.enabled = !config.enabled;
                    resp = resp.add_attribute("action", "toggle_status");
                }
                ExecuteMsg::UpdateConfig {
                    admins,
                    mint_frequency,
                    mint_amount,
                } => {
                    if let Some(admins) = admins {
                        config.admins = admins
                            .into_iter()
                            .map(|a| deps.api.addr_validate(&a))
                            .collect::<StdResult<Vec<_>>>()?;
                    }
                    if let Some(mint_frequency) = mint_frequency {
                        config.mint_frequency = mint_frequency;
                    }
                    if let Some(mint_amount) = mint_amount {
                        config.mint_amount = mint_amount;
                    }
                    resp = resp.add_attribute("action", "whitelist_admins");
                }
                ExecuteMsg::AddTokens(tokens) => {
                    let tokens: Result<Vec<_>, _> = tokens
                        .into_iter()
                        .map(|t| TokenData::into_valid(&env.contract.address, deps.as_ref(), t))
                        .collect();
                    let tokens = tokens?;
                    for token in tokens {
                        if !config.tokens.contains(&token) {
                            config.tokens.push(token);
                        }
                    }
                    resp = resp.add_attribute("action", "add_tokens");
                }
                _ => panic!("Code shouldn't reach here."),
            }
            CONFIG.save(deps.storage, &config)?;
            Ok(resp)
        }
    };
    pad_handle_result(resp, BLOCK_SIZE)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => to_binary(&config),
            QueryMsg::GetMintStatus(user) => {
                let user = deps.api.addr_validate(&user)?;
                let user_data = USER_DATA.may_load(deps.storage, &user)?.unwrap_or_default();
                let now = env.block.time.seconds();
                let time_since_last_mint = now - user_data.last_minted;
                if time_since_last_mint >= config.mint_frequency {
                    to_binary(&format!("{user} can mint tokens."))
                } else {
                    let cooldown = config.mint_frequency - time_since_last_mint;
                    to_binary(&format!(
                        "{user} must wait {cooldown} seconds before minting again."
                    ))
                }
            }
        },
        BLOCK_SIZE,
    )
}
