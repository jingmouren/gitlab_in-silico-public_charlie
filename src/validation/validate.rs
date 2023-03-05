use crate::validation::result::ValidationResult;
use std::collections::HashSet;

/// A trait that returns a vector of validation results
pub trait Validate {
    fn validate(&self) -> HashSet<ValidationResult>;
}
