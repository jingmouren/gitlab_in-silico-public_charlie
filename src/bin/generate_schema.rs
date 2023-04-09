use dropshot::ApiDescription;
use epictetus::env::{create_logger, get_openapi_schema_dir};
use epictetus::{allocate_endpoint, analyze_endpoint};
use serde_json::Value;
use slog::{info, Level, Logger};
use std::fs;
use std::process::Command;
use std::str::from_utf8;

fn main() {
    let logger = create_logger(Level::Info);

    let schema_file_path = get_openapi_schema_dir().join("openapi.json");
    info!(
        logger,
        "Will write OpenAPI schema to {:?}", schema_file_path
    );

    // Write the schema
    fs::write(
        schema_file_path,
        serde_json::to_string_pretty(&generate_schema(&logger))
            .expect("Failed to convert OpenAPI definition to pretty string."),
    )
    .expect("Failed to write the schema.");

    // Generate the index.html from the schema
    generate_index(&logger);

    info!(logger, "Done.");
}

/// Generates and returns the OpenAPI schema
fn generate_schema(logger: &Logger) -> Value {
    info!(
        logger,
        "Creating JSON schema for all input and output data structures..."
    );
    info!(logger, "Registering API endpoints.");
    let mut api = ApiDescription::new();
    api.register(allocate_endpoint).unwrap();
    api.register(analyze_endpoint).unwrap();

    info!(logger, "Generating OpenAPI JSON schema.");
    api.openapi("Epictetus", "v0")
        .json()
        .expect("Failed to convert OpenAPIDefinition to JSON.")
}

/// Generate index.html from the schema, by calling npx as a subprocess
fn generate_index(logger: &Logger) {
    info!(
        logger,
        "Generating index from schema in schema/openapi.json"
    );
    let mut generate_index_command = Command::new("npx");
    generate_index_command.args(vec![
        "@redocly/cli",
        "build-docs",
        "schema/openapi.json",
        "-o",
        "schema/index.html",
    ]);

    info!(
        logger,
        "Running npx command to generate index: {:?}", generate_index_command
    );
    let output = generate_index_command.output().unwrap();

    info!(
        logger,
        "Output of the npx command: {}",
        from_utf8(&output.stdout).unwrap()
    );
}

#[cfg(test)]
mod test {
    use crate::generate_schema;
    use epictetus::env::create_test_logger;
    use serde_json::Value;
    use std::fs;

    /// Fails if the newly generated schema is not the same as the one found inside the project.
    /// The test is useful for bringing awareness in schema changes, requiring manual update of the
    /// reference schema in the project when the schema changes.
    #[test]
    fn schema_change_monitor() {
        // Generate current schema
        let logger = create_test_logger();
        let json_schema = generate_schema(&logger);

        // Load reference schema
        let this_file_path =
            std::env::current_exe().expect("Can't get path of the current executable.");
        let schema_file_path = this_file_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("schema")
            .join("openapi.json");

        let reference_schema_str = fs::read_to_string(schema_file_path).unwrap();
        let reference_schema: Value = serde_json::from_str(reference_schema_str.as_str()).unwrap();

        // Assert that they are the same
        assert_eq!(json_schema, reference_schema, "Generated schema doesn't match the schema in ./schema/openapi.json. Did you forget to update the schema after changing the API?")
    }
}
