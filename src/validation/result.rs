type ErrorCode = &'static str;

/// Validation result can either be a Problem or Ok
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ValidationResult {
    PROBLEM(Problem),
    OK,
}

/// Validation problem with some basic information
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Problem {
    pub(crate) code: ErrorCode,
    pub(crate) message: String,
    pub(crate) severity: Severity,
}

/// Validation severity
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Severity {
    ERROR,
    WARNING,
}
