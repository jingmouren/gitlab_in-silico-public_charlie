use charlie::env::{create_logger, get_project_dir};
use reqwest::StatusCode;
use slog::{info, Level};
use std::fs;

/// Calls api endpoint on the localhost:8000 to get the schema
fn main() {
    let logger = create_logger(Level::Info);

    // Get the OpenAPI definition
    info!(logger, "Getting the OpenAPI schema.");
    let client = reqwest::blocking::Client::new();
    let response = client.get("http://localhost:8000/api").send().unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    info!(logger, "Getting the text representation of the response.");
    let json_schema_index = response.text().unwrap();

    info!(
        logger,
        "Asserting that the served index.html is the same as the one in schema dir."
    );

    // Load reference index.html
    let reference_index_file_path = get_project_dir().join("schema").join("index.html");

    let reference_index = fs::read_to_string(reference_index_file_path).unwrap();
    assert_eq!(json_schema_index, reference_index);

    info!(logger, "Done.");
}
