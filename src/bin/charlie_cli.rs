use charlie::env::create_logger;
use charlie::model::portfolio::{AllocationInput, Portfolio};
use charlie::model::responses::AllocationResult;
use charlie::{allocate, analyze};
use clap::Parser;
use slog::Level::Info;
use slog::{info, warn, Logger};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::str::FromStr;

/// Arguments to the command line interface.
#[derive(Parser)]
struct CliArgs {
    /// Action that we want to perform via the CLI.
    action: Action,
    /// Path to .yaml file that contains the input for the action.
    input_file_path: PathBuf,
}

/// Collections of actions exposed via the CLI.
#[derive(Clone)]
enum Action {
    Allocate,
    Analyze,
}

impl FromStr for Action {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "allocate" => Ok(Action::Allocate),
            "analyze" => Ok(Action::Analyze),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Expected \"allocate\" or \"analyze\" as action, got {}",
            )),
        }
    }
}

/// Deserializes the yaml content into the allocation input and performs the allocation.
fn allocate_action(logger: &Logger, yaml_file_content: String) {
    info!(
        logger,
        "Deserializing input file content to an AllocationInput object."
    );
    let input: AllocationInput = serde_yaml::from_str(&yaml_file_content.to_string()).unwrap();

    info!(logger, "Started calculating optimal portfolio allocation.");
    let portfolio: AllocationResult = match allocate(input, logger).error {
        None => p.0.result.unwrap(),
        Some(e) => panic!("{}", e.message),
    };
    let result = serde_yaml::to_string(&portfolio.allocations).unwrap();

    info!(logger, "Optimal portfolio is:\n{}", result);
}

/// Deserializes the yaml content into the analysis input and performs the analysis.
fn analyze_action(logger: &Logger, yaml_file_content: String) {
    info!(
        logger,
        "Deserializing input file content to a Portfolio object."
    );
    let input: Portfolio = serde_yaml::from_str(&yaml_file_content.to_string()).unwrap();

    info!(logger, "Analyzing the portfolio.");
    let analysis_result = match analyze(input, logger).0.error {
        None => r.0.result.unwrap(),
        Some(e) => panic!("{}", e.message),
    };
    let result = serde_yaml::to_string(&analysis_result).unwrap();

    info!(logger, "Portfolio statistics are:\n{}", result);
}

fn main() {
    let logger = create_logger(Info);
    info!(logger, "Parsing command line arguments...");
    let args: CliArgs = CliArgs::parse();
    let input_file_path: PathBuf = args.input_file_path;

    if input_file_path.extension().is_none() {
        warn!(
            logger,
            "Did not find the extension for the input file. Input file {} must be in yaml format.",
            input_file_path.display()
        );
    } else if input_file_path.extension().unwrap() != "yaml" {
        warn!(
            logger,
            "Input file's extension indicates that this might not be a .yaml file. Input file {} \
            must be in yaml format.",
            input_file_path.display()
        )
    }

    info!(logger, "Reading {} file.", input_file_path.display());
    let yaml_file_content: String = std::fs::read_to_string(&input_file_path)
        .expect("Did not manage to read file passed as an argument.");

    match args.action {
        Action::Allocate => {
            info!(logger, "Performing allocation.");
            allocate_action(&logger, yaml_file_content)
        }
        Action::Analyze => {
            info!(logger, "Performing portfolio analysis.");
            analyze_action(&logger, yaml_file_content)
        }
    }
}
