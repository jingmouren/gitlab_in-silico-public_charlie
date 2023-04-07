use dropshot::ApiDescription;
use portfolio::env::{create_logger, get_openapi_schema_dir};
use portfolio::{allocate_endpoint, analyze_endpoint};
use slog::{info, Level};
use std::fs;

fn main() {
    let logger = create_logger(Level::Info);
    info!(
        logger,
        "Creating JSON schema for all input and output data structures..."
    );

    let schema_file_path = get_openapi_schema_dir().join("openapi.json");

    info!(logger, "Registering API endpoints.");
    let mut api = ApiDescription::new();
    api.register(allocate_endpoint).unwrap();
    api.register(analyze_endpoint).unwrap();

    info!(
        logger,
        "Will write OpenAPI schema to {:?}", schema_file_path
    );
    fs::write(
        schema_file_path,
        serde_json::to_string_pretty(
            &api.openapi("Portfolio", "v0")
                .json()
                .expect("Failed to convert OpenAPIDefinition to JSON."),
        )
        .expect("Failed to convert OpenAPI definition to pretty string."),
    )
    .expect("Failed to write the schema.");

    info!(logger, "Done.");
}

// TODO: Add a test for schema generation
