use camino::Utf8PathBuf;
use dropshot::{
    ApiDescription, ConfigDropshot, ConfigLogging, ConfigLoggingIfExists, ConfigLoggingLevel,
    HttpServerStarter,
};
use portfolio::{allocate_endpoint, analyze_endpoint, openapi};
use slog::info;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Local host (127.0.0.1) with port 8000 (fixing port number for reproducibility in tests)
    let config_dropshot = ConfigDropshot {
        bind_address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8000)),
        request_body_max_bytes: 4096,
        tls: None,
    };

    // An "info"-level logger that writes to stderr assuming that it's a terminal.
    let config_logging = ConfigLogging::File {
        level: ConfigLoggingLevel::Info,
        path: Utf8PathBuf::from("./portfolio.log"),
        if_exists: ConfigLoggingIfExists::Append,
    };
    let log = config_logging
        .to_logger("portfolio")
        .map_err(|error| format!("failed to create logger: {}", error))?;

    // Create an API description object and register the endpoints
    info!(log, "Registering API endpoints.");
    let mut api = ApiDescription::new();
    api.register(openapi).unwrap();
    api.register(allocate_endpoint).unwrap();
    api.register(analyze_endpoint).unwrap();

    // Set up the server.
    info!(log, "Setting up the server.");
    let server = HttpServerStarter::new(&config_dropshot, api, (), &log)
        .map_err(|error| format!("failed to create server: {}", error))?
        .start();

    // Wait for the server to stop.  Note that there's not any code to shut down
    // this server, so we should never get past this point.
    info!(log, "Began serving.");
    server.await
}
