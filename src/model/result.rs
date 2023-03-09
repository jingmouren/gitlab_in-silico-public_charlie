use crate::model::company::Ticker;
use serde::{Deserialize, Serialize};

/// Complete result includes both the allocation result and the analysis of the resulting portfolio
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompleteResult {
    pub(crate) resulting_portfolio: AllocationResult,
    pub(crate) resulting_analysis: AnalysisResult,
}

/// Allocation result includes companies (tickers) and their fractions
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AllocationResult {
    pub(crate) allocations: Vec<TickerAndFraction>,
}

/// Analysis result includes some statistics for a given portfolio
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnalysisResult {
    pub(crate) worst_case_outcome: ProbabilityAndReturn,
    pub(crate) cumulative_probability_of_loss: f64,
    pub(crate) expected_value: f64,
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
