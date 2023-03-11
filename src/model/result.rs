use crate::model::company::Ticker;
use crate::validation::result::ValidationResult;
use serde::{Deserialize, Serialize};

/// All results contained inside a response to the client
/// TODO: Add expected errors here as well
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseResult {
    pub(crate) allocations: Option<Vec<TickerAndFraction>>,
    pub(crate) analysis: Option<AnalysisResult>,
    pub(crate) validation_errors: Option<Vec<ValidationResult>>,
}

/// Analysis result includes some statistics for a given portfolio
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnalysisResult {
    pub(crate) worst_case_outcome: ProbabilityAndReturn,
    pub(crate) cumulative_probability_of_loss: f64,
    pub(crate) expected_return: f64,
}

/// A ticker and a fraction used for minimalistic representation of the allocation calculation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TickerAndFraction {
    pub(crate) ticker: Ticker,
    pub(crate) fraction: f64,
}

/// Probability and return used to minimally represent an outcome
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProbabilityAndReturn {
    pub(crate) probability: f64,
    pub(crate) weighted_return: f64,
}
