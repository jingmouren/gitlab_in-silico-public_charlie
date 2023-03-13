use crate::model::company::Ticker;
use crate::validation::result::ValidationResult;
use serde::{Deserialize, Serialize};

/// All results contained inside a response to the client
/// TODO: Add expected errors here as well
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseResult {
    pub allocations: Option<Vec<TickerAndFraction>>,
    pub analysis: Option<AnalysisResult>,
    pub validation_errors: Option<Vec<ValidationResult>>,
}

/// Analysis result includes some statistics for a given portfolio
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnalysisResult {
    pub worst_case_outcome: ProbabilityAndReturn,
    pub cumulative_probability_of_loss: f64,
    pub expected_return: f64,
}

/// A ticker and a fraction used for minimalistic representation of the allocation calculation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TickerAndFraction {
    pub ticker: Ticker,
    pub fraction: f64,
}

/// Probability and return used to minimally represent an outcome
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProbabilityAndReturn {
    pub probability: f64,
    pub weighted_return: f64,
}
