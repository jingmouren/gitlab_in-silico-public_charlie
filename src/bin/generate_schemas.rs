use log::info;
use portfolio::model::portfolio::{Portfolio, PortfolioCandidates};
use portfolio::model::responses::{AllocationResponse, AnalysisResponse};
use schemars::schema_for;
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
    let schemas_dir = project_root_dir.join("schemas");
    info!("Will write all schemas to {:?}", schemas_dir);

    // PortfolioCandidates
    let schema_file = schemas_dir.join("portfolio_candidates_schema.json");
    info!(
        "Writing JSON schema for PortfolioCandidates to {:?}",
        schema_file
    );
    let schema = schema_for!(PortfolioCandidates);
    fs::write(schema_file, serde_json::to_string_pretty(&schema).unwrap()).unwrap();

    // Portfolio
    let schema_file = schemas_dir.join("portfolio_schema.json");
    info!("Writing JSON schema for Portfolio to {:?}", schema_file);
    let schema = schema_for!(Portfolio);
    fs::write(schema_file, serde_json::to_string_pretty(&schema).unwrap()).unwrap();

    // AllocationResponse
    let schema_file = schemas_dir.join("allocation_response_schema.json");
    info!(
        "Writing JSON schema for AllocationResponse to {:?}",
        schema_file
    );
    let schema = schema_for!(AllocationResponse);
    fs::write(schema_file, serde_json::to_string_pretty(&schema).unwrap()).unwrap();

    // AnalysisResponse
    let schema_file = schemas_dir.join("analysis_response_schema.json");
    info!(
        "Writing JSON schema for AnalysisResponse to {:?}",
        schema_file
    );
    let schema = schema_for!(AnalysisResponse);
    fs::write(schema_file, serde_json::to_string_pretty(&schema).unwrap()).unwrap();
}
