use shade_oracles::{
    Contract,
    interfaces::router::Config,
    storage::{Item, Map},
};

pub const CONFIG: Item<Config> = Item::new("YteGsgSZyO");
pub const ORACLES: Map<String, Contract> = Map::new("d3a17d1b");
pub const ALIASES: Map<String, String> = Map::new("iaunwdioafj");
