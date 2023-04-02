extern crate core;

pub mod allocation;
pub mod analysis;
pub mod env;
pub mod model;
pub mod validation;

use crate::allocation::{kelly_allocate, MAX_ITER};
use crate::analysis::{all_outcomes, worst_case_outcome};
use crate::analysis::{cumulative_probability_of_loss, expected_return};
use crate::env::get_openapi_schema_dir;
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
use dropshot::{endpoint, HttpError, HttpResponseOk, RequestContext, TypedBody};
use http::{Response, StatusCode};
use hyper::Body;
use slog::{info, Logger};
use std::collections::HashSet;
use std::fs;

/// OpenAPI documentation
#[endpoint {
    method = GET,
    path = "/api",
    tags = [ "api" ]
}]
pub async fn openapi(_rqctx: RequestContext<()>) -> Result<Response<Body>, HttpError> {
    let index_file_path = get_openapi_schema_dir().join("index.html");
    let index = fs::read_to_string(index_file_path.clone()).unwrap_or_else(|_| {
        panic!(
            "Did not manage to read index file at: {:?}",
            index_file_path
        )
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/html")
        .body(index.into())?)
}

/// Calculates optimal allocation that maximizes long-term growth of assets for a set of candidate
/// companies. Incorporates a small margin of safety by:
/// - Disallowing companies with negative expected return (would result in shorting),
/// - Disallowing use of leverage (i.e. the sum of all resulting fractions does not exceed 1).
/// Of course, the biggest margin of safety should come from the conservative assumptions in the
/// input data.
#[endpoint {
    method = POST,
    path = "/allocate",
    tags = [ "allocate" ],
}]
pub async fn allocate_endpoint(
    rqctx: RequestContext<()>,
    body: TypedBody<PortfolioCandidates>,
) -> Result<HttpResponseOk<AllocationResponse>, HttpError> {
    allocate(body.into_inner(), &rqctx.log)
}

/// Analyze the porftolio by calculating useful statistics
#[endpoint {
    method = POST,
    path = "/analyze",
    tags = [ "analyze" ],
}]
pub async fn analyze_endpoint(
    rqctx: RequestContext<()>,
    body: TypedBody<Portfolio>,
) -> Result<HttpResponseOk<AnalysisResponse>, HttpError> {
    analyze(body.into_inner(), &rqctx.log)
}

/// Validate the candidates and return all problematic validations
pub fn validate(
    portfolio_candidates: &PortfolioCandidates,
    logger: &Logger,
) -> Vec<ValidationResult> {
    info!(logger, "Performing validation of portfolio candidates.");
    let mut all_validation_results: HashSet<ValidationResult> = HashSet::new();

    portfolio_candidates
        .companies
        .iter()
        .for_each(|c| all_validation_results.extend(c.validate()));

    // Remove OK validation result and return
    let validation_problems: Vec<ValidationResult> = all_validation_results
        .into_iter()
        .filter(|vr| vr != &ValidationResult::OK)
        .collect();

    info!(
        logger,
        "Found {} validation problems.",
        validation_problems.len()
    );

    validation_problems
}

/// Calculates optimal allocation for each candidate company
pub fn allocate(
    portfolio_candidates: PortfolioCandidates,
    logger: &Logger,
) -> Result<HttpResponseOk<AllocationResponse>, HttpError> {
    info!(logger, "Started allocation.");

    // Return immediately if there is at least one validation error
    let validation_problems: Vec<ValidationResult> = validate(&portfolio_candidates, logger);
    if validation_problems.iter().any(|v| match v {
        ValidationResult::PROBLEM(p) => p.severity == ERROR,
        ValidationResult::OK => false,
    }) {
        info!(logger, "Validation problems found, returning them.");
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
    info!(
        logger,
        "Start filtering candidates that would produce undesirable results."
    );
    let mut filtered_candidates: Vec<Company> = vec![];
    portfolio_candidates.companies.into_iter().for_each(|c| {
        let downside_validation = c.validate_no_downside_scenario();
        match &downside_validation {
            ValidationResult::PROBLEM(problem) => info!(logger, "{}", problem.message),
            ValidationResult::OK => (),
        }

        let negative_expected_return_validation = c.validate_negative_expected_return();
        match &negative_expected_return_validation {
            ValidationResult::PROBLEM(problem) => info!(logger, "{}", problem.message),
            ValidationResult::OK => (),
        }

        // If both of these validation are ok, add the company
        if downside_validation == ValidationResult::OK
            && negative_expected_return_validation == ValidationResult::OK
        {
            filtered_candidates.push(c)
        } else {
            info!(logger, "Filtered out candidate {} because it either has a negative expected return or it doesn't have any downside.", c.ticker)
        }
    });

    // Return if there are no companies after filtering
    if filtered_candidates.is_empty() {
        info!(
            logger,
            "No valid candidates found after filtering, returning an error."
        );
        return Ok(HttpResponseOk(AllocationResponse {
            result: None,
            validation_problems: Some(validation_problems),
            error: Some(Error {
                code: "no-valid-candidates-for-allocation".to_string(),
                message: "Found no valid candidates for allocation. Check your input.".to_string(),
            }),
        }));
    }

    info!(
        logger,
        "Calculating the optimal allocation for {} candidates.",
        filtered_candidates.len()
    );
    let portfolio = match kelly_allocate(filtered_candidates, MAX_ITER, logger) {
        Ok(p) => p,
        Err(e) => {
            return Ok(HttpResponseOk(AllocationResponse {
                result: None,
                validation_problems: Some(validation_problems),
                error: Some(e),
            }));
        }
    };

    info!(logger, "Allocation complete, collecting allocation result.");
    let allocation_result: Vec<TickerAndFraction> = portfolio
        .companies
        .iter()
        .map(|pc| TickerAndFraction {
            ticker: pc.company.ticker.clone(),
            fraction: pc.fraction,
        })
        .collect();

    info!(
        logger,
        "Getting all outcomes in order to calculate some statistics about the portfolio."
    );
    let all_outcomes = match all_outcomes(&portfolio) {
        Ok(o) => o,
        Err(e) => {
            info!(
                logger,
                "Encountered an error while getting all outcomes. Returning it."
            );
            return Ok(HttpResponseOk(AllocationResponse {
                result: None,
                validation_problems: None,
                error: Some(e),
            }));
        }
    };
    let worst_case = worst_case_outcome(&all_outcomes, logger);

    info!(
        logger,
        "Allocation and analysis finished. Returning the allocation and analysis results."
    );
    Ok(HttpResponseOk(AllocationResponse {
        result: Some(AllocationResult {
            allocations: allocation_result,
            analysis: AnalysisResult {
                worst_case_outcome: ProbabilityAndReturn {
                    probability: worst_case.probability,
                    weighted_return: worst_case.weighted_return,
                },
                cumulative_probability_of_loss: cumulative_probability_of_loss(
                    &all_outcomes,
                    logger,
                ),
                expected_return: expected_return(&portfolio, logger),
            },
        }),
        validation_problems: Some(validation_problems),
        error: None,
    }))
}

/// Calculates useful information about the porftolio
pub fn analyze(
    portfolio: Portfolio,
    logger: &Logger,
) -> Result<HttpResponseOk<AnalysisResponse>, HttpError> {
    info!(
        logger,
        "Started portfolio analysis by getting all outcomes."
    );
    let all_outcomes = match all_outcomes(&portfolio) {
        Ok(o) => o,
        Err(e) => {
            info!(
                logger,
                "Encountered an error while getting all outcomes. Returning it."
            );
            return Ok(HttpResponseOk(AnalysisResponse {
                result: None,
                error: Some(e),
            }));
        }
    };
    let worst_case = worst_case_outcome(&all_outcomes, logger);

    info!(logger, "Analysis complete, returning.");
    Ok(HttpResponseOk(AnalysisResponse {
        result: Some(AnalysisResult {
            worst_case_outcome: ProbabilityAndReturn {
                probability: worst_case.probability,
                weighted_return: worst_case.weighted_return,
            },
            cumulative_probability_of_loss: cumulative_probability_of_loss(&all_outcomes, logger),
            expected_return: expected_return(&portfolio, logger),
        }),
        error: None,
    }))
}
