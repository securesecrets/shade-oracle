use shade_oracles::interfaces::index_oracle::{ExecuteMsg, InstantiateMsg, QueryMsg};
use shade_oracles::core::cosmwasm_schema;
use shade_oracles::core::cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
