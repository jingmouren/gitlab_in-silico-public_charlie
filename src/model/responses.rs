use crate::model::company::Ticker;
use crate::model::errors::Error;
use crate::validation::result::ValidationResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Response of the call to the allocate endpoint, contains results of both allocation and analysis.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct AllocationResponse {
    pub result: Option<AllocationResult>,
    pub validation_problems: Option<Vec<ValidationResult>>,
    pub error: Option<Error>,
}

/// Response of the call to the analyze endpoint.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct AnalysisResponse {
    pub result: Option<AnalysisResult>,
    pub error: Option<Error>,
}

/// Allocation result includes tickers and their fractions.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct AllocationResult {
    pub allocations: Vec<TickerAndFraction>,
    pub analysis: AnalysisResult,
}

/// Analysis result includes some statistics for a given portfolio.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct AnalysisResult {
    pub worst_case_outcome: ProbabilityAndReturns,
    pub cumulative_probability_of_loss: f64,
    pub expected_return: f64,
}

/// A ticker and a fraction used for minimalistic representation of the allocation calculation.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct TickerAndFraction {
    pub ticker: Ticker,
    pub fraction: f64,
}

/// Probability and returns used to minimally represent an outcome.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProbabilityAndReturns {
    pub probability: f64,
    pub portfolio_return: f64,
    pub probability_weighted_return: f64,
}
