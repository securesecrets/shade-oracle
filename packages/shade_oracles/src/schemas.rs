use ::std::{
    error::Error as StdError,
    env::current_dir,
    fs::{create_dir_all, write, create_dir, read_dir},
    process::Command,
};
use anyhow::Result;
use cosmwasm_schema::{Api, generate_api};
use shade_oracles::interfaces::{
    bot,
    index::msg as index,
    router::msg as router,
    derivatives::bot as derivatives_bot,
    derivatives::generic as derivatives_generic, providers,
    dex::generic as dex_generic,
};

const ROOT_DIR: &str = "schemas";

fn create_schema(name: &str, api: Api) {
    let directory = format!("{}/{}", ROOT_DIR, name);
    if let Err(e) = create_dir(&directory) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            eprintln!("Failed to create directory: {}", e);
            return;
        }
    }
    let file = format!("{}/{}.json", directory, name);
    write(&file, api.render().to_string().unwrap()).unwrap();
    println!("Exported the schema for {} as {}", name, file);
}

fn get_schema_directories() -> Result<Vec<String>> {
    let mut dirs = vec![];
    let dir_entries = read_dir(ROOT_DIR)?;
    for entry in dir_entries {
        let dir_entry = entry?;
        if dir_entry.file_type()?.is_dir() {
            let dir_name = dir_entry.file_name().into_string().unwrap();
            println!("{}", dir_name);
            dirs.push(dir_name);
        }
    }
    Ok(dirs)
}

fn run_codegen(schema_name: &str) {
    let output = Command::new("cosmwasm-ts-codegen")
        .args(["generate", "--typesOnly", "--schema"])
        .arg(format!("{}/{}", ROOT_DIR, schema_name))
        .args(["--out", "./types", "--name"])
        .arg(schema_name)
        .arg("--no-bundle")
        .output()
        .expect("Failed to run cosmwasm-ts-codegen");
    //println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("Generated types for {}", schema_name);
}

macro_rules! generate_and_push_api {
    ($apis:ident, $name:expr, $module:expr) => {
        $apis.push(cosmwasm_schema::generate_api!(
            name: $name,
            instantiate: $module::InstantiateMsg,
            query: $module::QueryMsg,
            execute: $module::ExecuteMsg,
        ));
    };
}

fn main() -> Result<()> {
    let mut out_dir = current_dir().unwrap();
    out_dir.push(ROOT_DIR);
    create_dir_all(&out_dir).unwrap();
    cosmwasm_schema::remove_schemas(&out_dir).unwrap();

    let mut apis = vec![];
    generate_and_push_api!(apis, "IndexOracle", index);
    generate_and_push_api!(apis, "BotOracle", bot);
    generate_and_push_api!(apis, "OracleRouter", router);
    generate_and_push_api!(apis, "DerivativesBot", derivatives_bot);
    generate_and_push_api!(apis, "DerivativesGeneric", derivatives_generic);
    generate_and_push_api!(apis, "DexGeneric", dex_generic);
    apis.push(generate_api!(
        name: "MockBandProvider",
        instantiate: providers::mock::BandInstantiateMsg,
        query: providers::BandQueryMsg,
        execute: providers::mock::BandExecuteMsg,
    ));
    apis.push(generate_api!(
        name: "MockOjoProvider",
        instantiate: providers::mock::OjoInstantiateMsg,
        query: providers::OjoQueryMsg,
        execute: providers::mock::OjoExecuteMsg,
    ));
    for api in apis {
        create_schema(&api.contract_name.clone(), api);
    }
    let schema_directories = get_schema_directories()?;
    for schema_directory in schema_directories {
        run_codegen(&schema_directory);
    }
    Ok(())
}
