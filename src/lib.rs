#![feature(decl_macro)]

mod allocation;
mod analysis;
pub mod api;
pub mod model;
pub mod validation;

use crate::allocation::{kelly_allocate, MAX_ITER};
use crate::analysis::{all_outcomes, worst_case_outcome};
use crate::analysis::{cumulative_probability_of_loss, expected_return};
use crate::model::company::Company;
use crate::model::portfolio::{Portfolio, PortfolioCandidates};
use crate::model::result::{
    AllocationResult, AnalysisResult, CompleteResult, ProbabilityAndReturn, TickerAndFraction,
};
use crate::validation::result::ValidationResult;
use crate::validation::validate::Validate;
use rocket::post;
use rocket::response::Responder;
use rocket_contrib::json::Json;
use std::collections::HashSet;

/// TODO
///  - Figure out the issue with Responder
///  - Move all these data types into model
///  - Re-introduce integration tests

/// Creates a vector of candidate companies from YAML
pub fn create_candidates(yaml_string: &str) -> Vec<Company> {
    // Deserialize candidates from yaml (TODO: Missing error handling)
    let candidates: Vec<Company> = serde_yaml::from_str(yaml_string).unwrap();

    // Collect all validation errors
    let mut all_validation_errors: HashSet<ValidationResult> = HashSet::new();
    candidates
        .iter()
        .for_each(|c| all_validation_errors.extend(c.validate()));

    // Panic at the moment: TODO: Error handling
    if all_validation_errors
        .iter()
        .any(|vr| *vr != ValidationResult::OK)
    {
        panic!("Found validation errors: {all_validation_errors:?}");
    }

    candidates
}

/// Calculates optimal allocation for each candidate company
#[post(
    "/allocate",
    format = "application/json",
    data = "<portfolio_candidates>"
)]
pub fn allocate(portfolio_candidates: PortfolioCandidates) -> Json<CompleteResult> {
    // Collect all validation errors
    let mut all_validation_errors: HashSet<ValidationResult> = HashSet::new();
    portfolio_candidates
        .companies
        .iter()
        .for_each(|c| all_validation_errors.extend(c.validate()));

    // Panic at the moment: TODO: Error handling
    if all_validation_errors
        .iter()
        .any(|vr| *vr != ValidationResult::OK)
    {
        panic!("Found validation errors: {all_validation_errors:?}");
    }

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

    let allocation_result = AllocationResult {
        allocations: portfolio
            .portfolio_companies
            .iter()
            .map(|pc| TickerAndFraction {
                ticker: pc.company.ticker.clone(),
                fraction: pc.fraction,
            })
            .collect(),
    };

    Json(CompleteResult {
        resulting_portfolio: allocation_result,
        resulting_analysis: AnalysisResult {
            worst_case_outcome: ProbabilityAndReturn {
                probability: 0.0,
                weighted_return: 0.0,
            },
            cumulative_probability_of_loss: 0.0,
            expected_value: 0.0,
        },
    })
}

/// Calculates and prints useful information about the portfolio
pub fn analyse(portfolio: &Portfolio) {
    expected_return(portfolio);

    let all_outcomes = all_outcomes(portfolio);
    worst_case_outcome(&all_outcomes);
    cumulative_probability_of_loss(&all_outcomes);
}
