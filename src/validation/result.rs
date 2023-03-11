use serde::{Deserialize, Serialize};

type ValidationCode = String;

/// Validation result can either be a Problem or Ok
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum ValidationResult {
    PROBLEM(Problem),
    OK,
}

/// Validation problem with some basic information
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct Problem {
    pub(crate) code: ValidationCode,
    pub(crate) message: String,
    pub(crate) severity: Severity,
}

/// Validation severity
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum Severity {
    ERROR,
    WARNING,
}
