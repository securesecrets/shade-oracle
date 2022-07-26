use shade_oracles::interfaces::common::{OracleQuery, ExecuteMsg};
use shade_oracles::interfaces::lp::market::InstantiateMsg;
use shade_oracles::core::cosmwasm_schema;
use shade_oracles::core::cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: OracleQuery,
        execute: ExecuteMsg,
    }
}
