extern crate core;

pub mod analysis;
pub mod constraints;
pub mod env;
pub mod kelly_allocation;
pub mod model;
pub mod utils;
pub mod validation;

use crate::analysis::{all_outcomes, worst_case_outcome};
use crate::analysis::{cumulative_probability_of_loss, expected_return};
use crate::env::get_project_dir;
use crate::kelly_allocation::{KellyAllocator, MAX_ITER};
use crate::model::company::Company;
use crate::model::errors::Error;
use crate::model::portfolio::{AllocationInput, Portfolio};
use crate::model::responses::{
    AllocationResponse, AllocationResult, AnalysisResponse, AnalysisResult, TickerAndFraction,
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

/// Basic front-end for simple demonstration purposes
#[endpoint {
    method = GET,
    path = "/demo",
    tags = [ "demo" ]
}]
pub async fn demo(_rqctx: RequestContext<()>) -> Result<Response<Body>, HttpError> {
    let demo_file_path = get_project_dir().join("demo").join("demo.html");
    let demo = fs::read_to_string(demo_file_path.clone()).unwrap_or_else(|_| {
        panic!(
            "Did not manage to read demo.html file at: {:?}",
            demo_file_path
        )
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/html")
        .body(demo.into())?)
}

/// OpenAPI documentation
#[endpoint {
    method = GET,
    path = "/api",
    tags = [ "api" ]
}]
pub async fn openapi(_rqctx: RequestContext<()>) -> Result<Response<Body>, HttpError> {
    let index_file_path = get_project_dir().join("schema").join("index.html");
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

/// Calculate optimal allocation of capital for a set of candidate companies with an optional
/// constraint representing aversion to the permanent loss of capital
#[endpoint {
    method = POST,
    path = "/allocate",
    tags = [ "allocate" ],
}]
pub async fn allocate_endpoint(
    rqctx: RequestContext<()>,
    body: TypedBody<AllocationInput>,
) -> Result<HttpResponseOk<AllocationResponse>, HttpError> {
    let allocation_result = allocate(body.into_inner(), &rqctx.log);
    Ok(HttpResponseOk(allocation_result))
}

/// Analyze the portfolio by calculating useful statistics
#[endpoint {
    method = POST,
    path = "/analyze",
    tags = [ "analyze" ],
}]
pub async fn analyze_endpoint(
    rqctx: RequestContext<()>,
    body: TypedBody<Portfolio>,
) -> Result<HttpResponseOk<AnalysisResponse>, HttpError> {
    let analysis_result = analyze(body.into_inner(), &rqctx.log);
    Ok(HttpResponseOk(analysis_result))
}

/// Validate the candidates and return all problematic validations.
pub fn validate(portfolio_candidates: &AllocationInput, logger: &Logger) -> Vec<ValidationResult> {
    info!(logger, "Performing validation of portfolio candidates.");
    let all_validation_results: HashSet<ValidationResult> = portfolio_candidates.validate();

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
pub fn allocate(allocation_input: AllocationInput, logger: &Logger) -> AllocationResponse {
    info!(logger, "Started allocation.");

    // Return immediately if there is at least one validation error
    let validation_problems: Vec<ValidationResult> = validate(&allocation_input, logger);
    if validation_problems.iter().any(|v| match v {
        ValidationResult::PROBLEM(p) => p.severity == ERROR,
        ValidationResult::OK => false,
    }) {
        info!(logger, "Validation problems found, returning them.");
        return AllocationResponse {
            result: None,
            validation_problems: Some(validation_problems),
            error: None,
        };
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
    allocation_input.candidates.into_iter().for_each(|c| {
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
        return AllocationResponse {
            result: None,
            validation_problems: Some(validation_problems),
            error: Some(Error {
                code: "no-valid-candidates-for-allocation".to_string(),
                message: "Found no valid candidates for allocation. Check your input.".to_string(),
            }),
        };
    }

    info!(
        logger,
        "Calculating the optimal allocation for {} candidates.",
        filtered_candidates.len()
    );
    let mut kelly_allocator = KellyAllocator::new(logger, MAX_ITER);

    // Add constraints if present
    if allocation_input.long_only.unwrap_or(false) {
        kelly_allocator = kelly_allocator.with_long_only_constraints(filtered_candidates.len());
    }

    if allocation_input.max_permanent_loss_of_capital.is_some() {
        let lc = allocation_input.max_permanent_loss_of_capital.unwrap();
        kelly_allocator = kelly_allocator.with_maximum_permanent_loss_constraint(lc);
    }

    if allocation_input.max_individual_allocation.is_some() {
        let max_f = allocation_input.max_individual_allocation.unwrap();
        kelly_allocator = kelly_allocator
            .with_maximum_individual_allocation_constraint(filtered_candidates.len(), max_f);
    }

    if allocation_input.max_total_leverage_ratio.is_some() {
        let max_lr = allocation_input.max_total_leverage_ratio.unwrap();
        kelly_allocator = kelly_allocator
            .with_maximum_total_leverage_constraint(filtered_candidates.len(), max_lr);
    }

    let portfolio = match kelly_allocator.allocate(filtered_candidates) {
        Ok(p) => p,
        Err(e) => {
            return AllocationResponse {
                result: None,
                validation_problems: Some(validation_problems),
                error: Some(e),
            };
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
            return AllocationResponse {
                result: None,
                validation_problems: None,
                error: Some(e),
            };
        }
    };
    let worst_case = worst_case_outcome(&portfolio, logger);

    info!(
        logger,
        "Allocation and analysis finished. Returning the allocation and analysis results."
    );
    AllocationResponse {
        result: Some(AllocationResult {
            allocations: allocation_result,
            analysis: AnalysisResult {
                worst_case_outcome: worst_case,
                cumulative_probability_of_loss: cumulative_probability_of_loss(
                    &all_outcomes,
                    logger,
                ),
                expected_return: expected_return(&portfolio, logger),
            },
        }),
        validation_problems: Some(validation_problems),
        error: None,
    }
}

/// Calculates useful information about the portfolio
pub fn analyze(portfolio: Portfolio, logger: &Logger) -> AnalysisResponse {
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
            return AnalysisResponse {
                result: None,
                error: Some(e),
            };
        }
    };
    let worst_case = worst_case_outcome(&portfolio, logger);

    info!(logger, "Analysis complete, returning.");
    AnalysisResponse {
        result: Some(AnalysisResult {
            worst_case_outcome: worst_case,
            cumulative_probability_of_loss: cumulative_probability_of_loss(&all_outcomes, logger),
            expected_return: expected_return(&portfolio, logger),
        }),
        error: None,
    }
}
