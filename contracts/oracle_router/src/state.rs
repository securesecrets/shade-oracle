use serde::{Deserialize, Serialize};
use shade_oracles::{
    common::{Contract},
    scrt::{
        HumanAddr,
    }, storage::{Item, Map},
};

pub const CONFIG: Item<Config> = Item::new("YteGsgSZyO");
pub const ORACLES: Map<String, Contract> = Map::new("d3a17d1b");

#[derive(Serialize, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub owner: HumanAddr,
    pub default_oracle: Contract,
}
