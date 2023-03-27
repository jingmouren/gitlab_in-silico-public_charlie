pub mod allocation;
pub mod analysis;
pub mod model;
pub mod validation;

use crate::allocation::{kelly_allocate, MAX_ITER};
use crate::analysis::{all_outcomes, worst_case_outcome};
use crate::analysis::{cumulative_probability_of_loss, expected_return};
use dropshot::{endpoint, HttpError, HttpResponseOk, RequestContext, TypedBody};
use crate::model::company::Company;
use crate::model::errors::Error;
use crate::model::portfolio::{Portfolio, PortfolioCandidates};
use crate::model::responses::{
    AllocationResponse, AllocationResult, AnalysisResponse, AnalysisResult, ProbabilityAndReturn,
    TickerAndFraction,
};
use crate::validation::result::Severity::ERROR;
use crate::validation::result::ValidationResult;
use crate::validation::validate::Validate;
use std::collections::HashSet;
use log::info;

/// Validate the candidates and return all problematic validations
pub fn validate(portfolio_candidates: &PortfolioCandidates) -> Vec<ValidationResult> {
    let mut all_validation_results: HashSet<ValidationResult> = HashSet::new();

    portfolio_candidates
        .companies
        .iter()
        .for_each(|c| all_validation_results.extend(c.validate()));

    // Remove OK validation result and return
    all_validation_results
        .into_iter()
        .filter(|vr| vr != &ValidationResult::OK)
        .collect()
}

/// Calculates optimal allocation for each candidate company
#[endpoint {
    method = POST,
    path = "/allocate",
    tags = [ "allocate" ],
}]
pub async fn allocate(
    _rqctx: RequestContext<()>, // Not used but needed by dropshot's interface
    body: TypedBody<PortfolioCandidates>,
) -> Result<HttpResponseOk<AllocationResponse>, HttpError> {
    // Return immediately if there is at least one validation error
    let portfolio_candidates: PortfolioCandidates = body.into_inner();
    let validation_problems: Vec<ValidationResult> = validate(&portfolio_candidates);
    if validation_problems.iter().any(|v| match v {
        ValidationResult::PROBLEM(p) => p.severity == ERROR,
        ValidationResult::OK => false,
    }) {
        return Ok(HttpResponseOk(AllocationResponse {
            result: None,
            validation_problems: Some(validation_problems),
            error: None,
        }));
    }

    // Create a subset of all candidates that can be handled by the algorithm. We don't allow:
    // 1. Candidates that have a negative expected return (would result in shorting),
    // 2. Candidates that don't have any downside (would result in numerical failure because the
    //    mathematical solution is to put an infinite amount of levered money into it)
    let mut filtered_candidates: Vec<Company> = vec![];
    portfolio_candidates.companies.into_iter().for_each(|c| {
        let downside_validation = c.validate_no_downside_scenario();
        match &downside_validation {
            ValidationResult::PROBLEM(problem) => info!("{}", problem.message),
            ValidationResult::OK => (),
        }

        let negative_expected_return_validation = c.validate_negative_expected_return();
        match &negative_expected_return_validation {
            ValidationResult::PROBLEM(problem) => info!("{}", problem.message),
            ValidationResult::OK => (),
        }

        // If both of these validation are ok, add the company
        if downside_validation == ValidationResult::OK
            && negative_expected_return_validation == ValidationResult::OK
        {
            filtered_candidates.push(c)
        }
    });

    // Return if there are no companies after filtering
    if filtered_candidates.is_empty() {
        return Ok(HttpResponseOk(AllocationResponse {
            result: None,
            validation_problems: Some(validation_problems),
            error: Some(Error {
                code: "no-valid-candidates-for-allocation".to_string(),
                message: "Found no valid candidates for allocation. Check your input.".to_string(),
            }),
        }));
    }

    let portfolio = match kelly_allocate(filtered_candidates, MAX_ITER) {
        Ok(p) => p,
        Err(e) => {
            return Ok(HttpResponseOk(AllocationResponse {
                result: None,
                validation_problems: Some(validation_problems),
                error: Some(e),
            }));
        }
    };

    let allocation_result: Vec<TickerAndFraction> = portfolio
        .companies
        .iter()
        .map(|pc| TickerAndFraction {
            ticker: pc.company.ticker.clone(),
            fraction: pc.fraction,
        })
        .collect();

    let all_outcomes = match all_outcomes(&portfolio) {
        Ok(o) => o,
        Err(e) => {
            return Ok(HttpResponseOk(AllocationResponse {
                result: None,
                validation_problems: None,
                error: Some(e),
            }));
        }
    };
    let worst_case = worst_case_outcome(&all_outcomes);

    Ok(HttpResponseOk(AllocationResponse {
        result: Some(AllocationResult {
            allocations: allocation_result,
            analysis: AnalysisResult {
                worst_case_outcome: ProbabilityAndReturn {
                    probability: worst_case.probability,
                    weighted_return: worst_case.weighted_return,
                },
                cumulative_probability_of_loss: cumulative_probability_of_loss(&all_outcomes),
                expected_return: expected_return(&portfolio),
            },
        }),
        validation_problems: Some(validation_problems),
        error: None,
    }))
}

/// Calculates and prints useful information about the portfolio
#[endpoint {
    method = POST,
    path = "/analyze",
    tags = [ "analyze" ],
}]
pub async fn analyze(
    _rqctx: RequestContext<()>, // Not used but needed by dropshot's interface
    body: TypedBody<Portfolio>,
) -> Result<HttpResponseOk<AnalysisResponse>, HttpError> {
    let portfolio: Portfolio = body.into_inner();

    let all_outcomes = match all_outcomes(&portfolio) {
        Ok(o) => o,
        Err(e) => {
            return Ok(HttpResponseOk(AnalysisResponse {
                result: None,
                error: Some(e),
            }));
        }
    };
    let worst_case = worst_case_outcome(&all_outcomes);

    Ok(HttpResponseOk(AnalysisResponse {
        result: Some(AnalysisResult {
            worst_case_outcome: ProbabilityAndReturn {
                probability: worst_case.probability,
                weighted_return: worst_case.weighted_return,
            },
            cumulative_probability_of_loss: cumulative_probability_of_loss(&all_outcomes),
            expected_return: expected_return(&portfolio),
        }),
        error: None,
    }))
}
