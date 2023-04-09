use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

type ValidationCode = String;

/// Validation result can either be a Problem or Ok
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Clone, Debug)]
pub enum ValidationResult {
    PROBLEM(Problem),
    OK,
}

/// Validation problem with some basic information
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Clone, Debug)]
pub struct Problem {
    pub code: ValidationCode,
    pub message: String,
    pub severity: Severity,
}

/// Validation severity
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Clone, Debug)]
pub enum Severity {
    ERROR,
    WARNING,
}
