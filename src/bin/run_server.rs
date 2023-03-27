use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use dropshot::{ApiDescription, ConfigDropshot, ConfigLogging, ConfigLoggingLevel, HttpServerStarter};
use portfolio::{allocate, analyze};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Local host (127.0.0.1) with port 8000 (fixing port number for reproducibility in tests)
    let config_dropshot = ConfigDropshot {
        bind_address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8000)),
        request_body_max_bytes: 4096,
        tls: None,
    };

    // An "info"-level logger that writes to stderr assuming that it's a terminal.
    let config_logging =
        ConfigLogging::StderrTerminal { level: ConfigLoggingLevel::Info };
    let log = config_logging
        .to_logger("portfolio")
        .map_err(|error| format!("failed to create logger: {}", error))?;

    // Create an API description object and register the endpoints
    let mut api = ApiDescription::new();
    api.register(allocate).unwrap();
    api.register(analyze).unwrap();

    // Write the OpenAPI schema
    // api.openapi("Portfolio", "v1")
    //     .write(&mut std::io::stdout())
    //     .map_err(|e| e.to_string())?;

    // Set up the server.
    let server =
        HttpServerStarter::new(&config_dropshot, api, (), &log)
            .map_err(|error| format!("failed to create server: {}", error))?
            .start();

    // Wait for the server to stop.  Note that there's not any code to shut down
    // this server, so we should never get past this point.
    server.await
}
