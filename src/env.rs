use slog::{o, Drain, Logger};

/// Creates a logger object. Used in certain utilities and tests
pub fn create_logger() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    Logger::root(drain, o!())
}
