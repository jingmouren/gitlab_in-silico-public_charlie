use std::fs;
use epictetus::env::create_logger;
use reqwest::StatusCode;
use slog::{info, Level};

/// Calls api endpoint on the localhost:8000 to get the schema
fn main() {
    let logger = create_logger(Level::Info);

    // Get the OpenAPI definition
    info!(logger, "Getting the OpenAPI schema.");
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("http://localhost:8000/api")
        .send()
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    info!(logger, "Getting the text representation of the response.");
    let json_schema_index = response.text().unwrap();

    info!(logger, "Asserting that the served index.html is the same as the one in schema dir.");
    // Load reference index.html
    let this_file_path =
        std::env::current_exe().expect("Can't get path of the current executable.");
    let reference_index_file_path = this_file_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("schema")
        .join("index.html");

    let reference_index = fs::read_to_string(reference_index_file_path).unwrap();
    println!("{}", json_schema_index);
    println!("{}", reference_index);
    assert_eq!(json_schema_index, reference_index);

    info!(logger, "Done.");
}
