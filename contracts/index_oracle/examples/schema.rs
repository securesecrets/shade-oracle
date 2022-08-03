use shade_oracles::core::cosmwasm_schema;
use shade_oracles::core::cosmwasm_schema::write_api;
use shade_oracles::interfaces::index_oracle::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
