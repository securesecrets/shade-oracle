use shade_oracles::core::cosmwasm_schema;
use shade_oracles::core::cosmwasm_schema::write_api;
use shade_oracles::interfaces::common::{ExecuteMsg, OracleQuery};
use shade_oracles::interfaces::staking_derivative::shade::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: OracleQuery,
        execute: ExecuteMsg,
    }
}
