use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::{Contract},
    storage::{Item, Map},
};
use cosmwasm_std::{
    HumanAddr,
};

pub const CONFIG: Item<Config> = Item::new("YteGsgSZyO");
pub const ORACLES: Map<String, Contract> = Map::new("d3a17d1b");
pub const ALIASES: Map<String, String> = Map::new("iaunwdioafj");

#[derive(Serialize, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub owner: HumanAddr,
    pub default_oracle: Contract,
}
