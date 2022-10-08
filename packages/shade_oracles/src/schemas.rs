use ::std::{
    env::current_dir,
    fs::{create_dir_all, write},
};
use cosmwasm_schema::QueryResponses;
use shade_oracles::interfaces::*;

const TARGET_DIR: &str = "artifacts";

macro_rules! schema {
    ($dir:ident, $($name:literal, $msg:path),+) => {
        $({
            use $msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
            let api_object = cosmwasm_schema::generate_api! {
                name: $name,
                instantiate: InstantiateMsg,
                query: QueryMsg,
                execute: ExecuteMsg,
            };
            let api = api_object.render();

            write_schema!($dir, $name, api);
        })+
    };
    (generic_oracles, $dir:ident, $($name:literal, $msg:path),+) => {
        $({
            use shade_oracles::interfaces::common::{ExecuteMsg, OracleQuery};
            use $msg::{InstantiateMsg};
            let api_object = cosmwasm_schema::generate_api! {
                name: $name,
                instantiate: InstantiateMsg,
                query: OracleQuery,
                execute: ExecuteMsg,
            };
            let api = api_object.render();

            write_schema!($dir, $name, api);
        })+
    };
}

macro_rules! write_schema {
    ($dir:ident, $name:literal, $api:ident) => {
        let path = $dir.join(concat!($name, ".json"));
        let json = $api.to_string().unwrap();
        write(&path, json + "\n").unwrap();
        println!("Exported the API as {}", path.to_str().unwrap());
    };
}

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push(TARGET_DIR);
    create_dir_all(&out_dir).unwrap();
    cosmwasm_schema::remove_schemas(&out_dir).unwrap();

    schema!(
        out_dir,
        "index_oracle",
        index::msg,
        "mock_band",
        band,
        "oracle_router",
        router::msg
    );

    schema!(
        generic_oracles,
        out_dir,
        "shade_staking_derivative_oracle",
        staking_derivative::shade,
        "shadeswap_market_oracle",
        lp::market,
        "siennaswap_lp_oracle",
        lp::siennaswap,
        "siennaswap_lp_spot_oracle",
        lp::siennaswap,
        "siennaswap_market_oracle",
        lp::market
    );
}
