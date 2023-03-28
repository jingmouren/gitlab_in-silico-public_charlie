use dropshot::ApiDescription;
use log::info;
use portfolio::{allocate_endpoint, analyze_endpoint};
use simple_logger::SimpleLogger;
use std::{env, fs};

fn main() {
    SimpleLogger::new().init().unwrap();
    info!("Creating JSON schema for all input and output data structures...");

    let this_file_path = env::current_exe().expect("Can't get path of this file.");
    let project_root_dir = this_file_path
        .parent()
        .expect("Can't get first parent of this file.")
        .parent()
        .expect("Can't get second parent of this file.")
        .parent()
        .expect("Can't get third parent of this file");
    let schema_file = project_root_dir.join("schema").join("openapi_schema.json");

    info!("Registering API endpoints.");
    let mut api = ApiDescription::new();
    api.register(allocate_endpoint).unwrap();
    api.register(analyze_endpoint).unwrap();

    info!("Will OpenAPI schema to {:?}", schema_file);
    fs::write(
        schema_file,
        serde_json::to_string_pretty(
            &api.openapi("Portfolio", "v1")
                .json()
                .expect("Failed to convert OpenAPIDefinition to JSON."),
        )
        .expect("Failed to convert OpenAPI definition to pretty string."),
    )
    .expect("Failed to write the schema.");

    info!("Done.");
}
