#![feature(decl_macro)]

mod allocation;
mod analysis;
pub mod api;
pub mod model;
pub mod validation;

use crate::allocation::{kelly_allocate, MAX_ITER};
use crate::analysis::{all_outcomes, worst_case_outcome};
use crate::analysis::{cumulative_probability_of_loss, expected_return};
use crate::model::portfolio::{Portfolio, PortfolioCandidates};
use crate::model::result::{
    AnalysisResult, ProbabilityAndReturn, ResponseResult, TickerAndFraction,
};
use crate::validation::result::ValidationResult;
use crate::validation::validate::Validate;
use rocket::post;
use rocket_contrib::json::Json;
use std::collections::HashSet;

/// TODO
///  - Re-introduce integration tests

/// Creates a vector of candidate companies from YAML
pub fn create_candidates(yaml_string: &str) -> PortfolioCandidates {
    // Deserialize candidates from yaml (TODO: Missing error handling)
    let candidates: PortfolioCandidates = serde_yaml::from_str(yaml_string).unwrap();

    validate(&candidates);

    candidates
}

/// Validate the candidates and return all problematic validations
pub fn validate(portfolio_candidates: &PortfolioCandidates) -> Vec<ValidationResult> {
    let mut all_validation_errors: HashSet<ValidationResult> = HashSet::new();

    portfolio_candidates
        .companies
        .iter()
        .for_each(|c| all_validation_errors.extend(c.validate()));

    // Remove OK validation result and return
    all_validation_errors
        .into_iter()
        .filter(|vr| vr != &ValidationResult::OK)
        .collect()
}

/// Calculates optimal allocation for each candidate company
#[post(
    "/allocate",
    format = "application/json",
    data = "<portfolio_candidates>"
)]
pub fn allocate(portfolio_candidates: PortfolioCandidates) -> Json<ResponseResult> {
    // TODO: Distinguish between warnings and errors
    let validation_errors: Vec<ValidationResult> = validate(&portfolio_candidates);
    if !validation_errors.is_empty() {
        return Json(ResponseResult {
            allocations: None,
            analysis: None,
            validation_errors: Some(validation_errors),
        });
    }

    // TODO: This possibly belongs to validation warning
    // Retain only the candidates that have positive expected value. This would otherwise likely
    // lead to negative fractions (which implies shorting). Note that I said "likely" because I'm
    // not 100% sure, but just have a feeling.
    let filtered_candidates = portfolio_candidates
        .companies
        .iter()
        .cloned()
        .filter(|c| {
            c.scenarios
                .iter()
                .map(|s| s.probability * (s.intrinsic_value - c.market_cap) / c.market_cap)
                .sum::<f64>()
                > 0.0
        })
        .collect();

    // TODO:
    //  1. Add info statement for filtered candidates
    //  2. Filter also the "perfect candidates (the ones without any downside)

    let portfolio = kelly_allocate(filtered_candidates, MAX_ITER);

    let allocation_result: Vec<TickerAndFraction> = portfolio
        .portfolio_companies
        .iter()
        .map(|pc| TickerAndFraction {
            ticker: pc.company.ticker.clone(),
            fraction: pc.fraction,
        })
        .collect();

    let all_outcomes = all_outcomes(&portfolio);
    let worst_case = worst_case_outcome(&all_outcomes);

    Json(ResponseResult {
        allocations: Some(allocation_result),
        analysis: Some(AnalysisResult {
            worst_case_outcome: ProbabilityAndReturn {
                probability: worst_case.probability,
                weighted_return: worst_case.weighted_return,
            },
            cumulative_probability_of_loss: cumulative_probability_of_loss(&all_outcomes),
            expected_return: expected_return(&portfolio),
        }),
        validation_errors: None,
    })
}

/// Calculates and prints useful information about the portfolio
pub fn analyse(portfolio: &Portfolio) {
    expected_return(portfolio);

    let all_outcomes = all_outcomes(portfolio);
    worst_case_outcome(&all_outcomes);
    cumulative_probability_of_loss(&all_outcomes);
}
