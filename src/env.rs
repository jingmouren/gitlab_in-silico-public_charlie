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

/// Gets the project directory by looking for directory where Cargo.toml is located, starting from
/// the directory that contains the current executable file.
pub fn get_project_dir() -> PathBuf {
    let this_file_path =
        std::env::current_exe().expect("Can't get path of the current executable.");

    let mut project_dir = this_file_path
        .parent()
        .expect("Can't get first parent of the executable.");
    loop {
        if project_dir.is_dir() && project_dir.join("Cargo.toml").exists() {
            break;
        } else {
            project_dir = project_dir
                .parent()
                .expect("Couldn't find parent of directory.")
        }
    }

    project_dir.to_path_buf()
}

#[cfg(test)]
mod test {
    use crate::env::get_project_dir;

    #[test]
    fn test_get_project_dir() {
        assert!(get_project_dir().join("Cargo.toml").exists());
    }
}
