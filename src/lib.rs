pub mod allocation;
pub mod analysis;
pub mod model;
pub mod validation;

use crate::allocation::{kelly_allocate, MAX_ITER};
use crate::analysis::{all_outcomes, worst_case_outcome};
use crate::analysis::{cumulative_probability_of_loss, expected_return};
use crate::model::portfolio::{Portfolio, PortfolioCandidates};
use crate::model::result::{
    AllocationResult, AnalysisResult, ProbabilityAndReturn, TickerAndFraction,
};
use crate::validation::result::ValidationResult;
use crate::validation::validate::Validate;
use rocket::post;
use rocket::serde::json::Json;
use std::collections::HashSet;

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
pub fn allocate(portfolio_candidates: PortfolioCandidates) -> Json<AllocationResult> {
    // TODO: Distinguish between warnings and errors
    let validation_errors: Vec<ValidationResult> = validate(&portfolio_candidates);
    if !validation_errors.is_empty() {
        return Json(AllocationResult {
            allocations: None,
            analysis: None,
            validation_errors: Some(validation_errors),
            errors: None,
            warnings: None,
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

    let portfolio = match kelly_allocate(filtered_candidates, MAX_ITER) {
        Ok(p) => p,
        Err(e) => {
            return Json(AllocationResult {
                allocations: None,
                analysis: None,
                validation_errors: None,
                errors: Some(e),
                warnings: None,
            })
        }
    };

    let allocation_result: Vec<TickerAndFraction> = portfolio
        .portfolio_companies
        .iter()
        .map(|pc| TickerAndFraction {
            ticker: pc.company.ticker.clone(),
            fraction: pc.fraction,
        })
        .collect();

    let all_outcomes = match all_outcomes(&portfolio) {
        Ok(o) => o,
        Err(e) => {
            return Json(AllocationResult {
                allocations: None,
                analysis: None,
                validation_errors: None,
                errors: Some(e),
                warnings: None,
            })
        }
    };
    let worst_case = worst_case_outcome(&all_outcomes);

    Json(AllocationResult {
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
        errors: None,
        warnings: None,
    })
}

/// Calculates and prints useful information about the portfolio
#[post("/analyze", format = "application/json", data = "<portfolio>")]
pub fn analyze(portfolio: Portfolio) -> Json<AnalysisResult> {
    let all_outcomes = match all_outcomes(&portfolio) {
        Ok(o) => o,
        // TODO: Return an error in the response
        Err(_e) => {
            return Json(AnalysisResult {
                worst_case_outcome: ProbabilityAndReturn {
                    probability: 0.0,
                    weighted_return: 0.0,
                },
                cumulative_probability_of_loss: 0.0,
                expected_return: 0.0,
            })
        }
    };
    let worst_case = worst_case_outcome(&all_outcomes);

    Json(AnalysisResult {
        worst_case_outcome: ProbabilityAndReturn {
            probability: worst_case.probability,
            weighted_return: worst_case.weighted_return,
        },
        cumulative_probability_of_loss: cumulative_probability_of_loss(&all_outcomes),
        expected_return: expected_return(&portfolio),
    })
}
