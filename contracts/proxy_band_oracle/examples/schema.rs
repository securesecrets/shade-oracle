use shade_oracles::core::cosmwasm_schema;
use shade_oracles::core::cosmwasm_schema::write_api;
use shade_oracles::interfaces::band::proxy::InstantiateMsg;
use shade_oracles::interfaces::common::{ExecuteMsg, OracleQuery};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: OracleQuery,
        execute: ExecuteMsg,
    }
}
