use slog::{o, Drain, Level, Logger};
use slog_async::OverflowStrategy;
use std::path::PathBuf;

/// Creates a logger object. Used in certain utilities and tests
pub fn create_logger(level: Level) -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator)
        .build()
        .filter_level(level)
        .fuse();
    let drain = slog_async::Async::new(drain)
        .overflow_strategy(OverflowStrategy::Block)
        .build()
        .fuse();
    Logger::root(drain, o!())
}

/// Test logger filters out everything below Warning level. For debugging tests, change to Debug
pub fn create_test_logger() -> Logger {
    create_logger(Level::Warning)
}

/// Gets schema directory which is in PROJECT_DIR/schema/openapi_schema.json
/// Assumes that the executable calling this function is in PROJECT_DIR/src/bin
pub fn get_openapi_schema_dir() -> PathBuf {
    let this_file_path =
        std::env::current_exe().expect("Can't get path of the current executable.");
    let project_root_dir = this_file_path
        .parent()
        .expect("Can't get first parent of this file.")
        .parent()
        .expect("Can't get second parent of this file.")
        .parent()
        .expect("Can't get third parent of this file");
    project_root_dir.join("schema")
}
