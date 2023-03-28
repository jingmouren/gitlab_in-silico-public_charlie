use slog::{o, Drain, Logger};
use std::path::PathBuf;

/// Creates a logger object. Used in certain utilities and tests
pub fn create_logger() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    Logger::root(drain, o!())
}

/// Gets schema file path which is in PROJECT_DIR/schema/openapi_schema.json
/// Assumes that the executable calling this function is in PROJECT_DIR/src/bin
pub fn get_schema_file_path() -> PathBuf {
    let this_file_path =
        std::env::current_exe().expect("Can't get path of the current executable.");
    let project_root_dir = this_file_path
        .parent()
        .expect("Can't get first parent of this file.")
        .parent()
        .expect("Can't get second parent of this file.")
        .parent()
        .expect("Can't get third parent of this file");
    project_root_dir.join("schema").join("openapi_schema.json")
}
