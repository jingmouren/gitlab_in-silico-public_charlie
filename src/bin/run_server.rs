use camino::Utf8PathBuf;
use charlie::env::get_project_dir;
use charlie::{allocate_endpoint, analyze_endpoint, demo, openapi};
use dropshot::{
    ApiDescription, ConfigDropshot, ConfigLogging, ConfigLoggingIfExists, ConfigLoggingLevel,
    HttpServerStarter,
};
use slog::info;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), String> {
    // Read file containing the server config
    let server_config_file_path = get_project_dir().join("server_config.toml");
    let server_config_str =
        fs::read_to_string(server_config_file_path.clone()).unwrap_or_else(|_| {
            panic!(
                "Did not manage to read server config file at: {:?}",
                server_config_file_path
            )
        });
    let server_config: ConfigDropshot =
        toml::from_str(&server_config_str).expect("Failed to deserialize server config.");

    // An info-level logger to a file
    let config_logging = ConfigLogging::File {
        level: ConfigLoggingLevel::Info,
        path: Utf8PathBuf::from("./server.log"),
        if_exists: ConfigLoggingIfExists::Append,
    };
    let log = config_logging
        .to_logger("charlie")
        .map_err(|error| format!("failed to create logger: {}", error))?;

    // Create an API description object and register the endpoints
    info!(log, "Registering API endpoints.");
    let mut api = ApiDescription::new();
    api.register(openapi).unwrap();
    api.register(allocate_endpoint).unwrap();
    api.register(analyze_endpoint).unwrap();
    api.register(demo).unwrap();

    // Set up the server.
    info!(log, "Setting up the server.");
    let server = HttpServerStarter::new(&server_config, api, (), &log)
        .map_err(|error| format!("failed to create server: {}", error))?
        .start();

    // Wait for the server to stop.  Note that there's not any code to shut down
    // this server, so we should never get past this point.
    info!(log, "Began serving.");
    server.await
}
