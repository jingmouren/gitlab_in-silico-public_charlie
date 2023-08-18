use charlie::env::{create_logger, create_test_logger, get_project_dir};
use charlie::kelly_allocation::{KellyAllocator, MAX_ITER, SOLVER_TOLERANCE};
use charlie::model::errors::Error;
use charlie::model::portfolio::AllocationInput;
use charlie::model::responses::{AllocationResponse, AnalysisResponse, TickerAndFraction};
use charlie::utils::assert_close;
use charlie::validation::result::{Problem, Severity, ValidationResult};
use charlie::{allocate, analyze, validate};
use slog::info;
use slog::Level::Info;

/// Make assertion tolerance the same as the fraction tolerance (no point in more accuracy).
const ASSERTION_TOLERANCE: f64 = SOLVER_TOLERANCE;

/// Helper function to load the test YAML file as a string.
fn load_test_input_file() -> String {
    let test_file_path = get_project_dir().join("tests").join("test_data.yaml");
    std::fs::read_to_string(&test_file_path)
        .expect("Did not manage to read test file in PROJECT_DIR/tests/test_data.yaml.")
}

#[test]
fn test_create_candidates_and_validate() {
    let input: AllocationInput = serde_yaml::from_str(&load_test_input_file()).unwrap();

    assert_eq!(input.candidates.len(), 6);

    // First company
    assert_eq!(input.candidates[0].name, "A");
    assert_eq!(input.candidates[0].ticker, "A");
    assert_eq!(input.candidates[0].description, "Business A");
    assert_eq!(input.candidates[0].market_cap, 238.0e9);

    // Scenarios for the first company
    assert_eq!(input.candidates[0].scenarios.len(), 4);
    assert_eq!(
        input.candidates[0].scenarios[0].thesis,
        "Unexpected stuff happens"
    );
    assert_eq!(input.candidates[0].scenarios[0].intrinsic_value, 0.0);
    assert_eq!(input.candidates[0].scenarios[0].probability, 0.05);

    assert_eq!(
        input.candidates[0].scenarios[1].thesis,
        "Core business keeps losing earnings power"
    );
    assert_eq!(input.candidates[0].scenarios[1].intrinsic_value, 170.0e9);
    assert_eq!(input.candidates[0].scenarios[1].probability, 0.3);

    assert_eq!(
        input.candidates[0].scenarios[2].thesis,
        "Business doesn't grow, earnings kept flat"
    );
    assert_eq!(input.candidates[0].scenarios[2].intrinsic_value, 270.0e9);
    assert_eq!(input.candidates[0].scenarios[2].probability, 0.5);

    assert_eq!(
        input.candidates[0].scenarios[3].thesis,
        "Earnings grow slightly"
    );
    assert_eq!(input.candidates[0].scenarios[3].intrinsic_value, 360.0e9);
    assert_eq!(input.candidates[0].scenarios[3].probability, 0.15);

    // Last company
    assert_eq!(input.candidates[5].name, "F");
    assert_eq!(input.candidates[5].ticker, "F");
    assert_eq!(input.candidates[5].description, "Business F");
    assert_eq!(input.candidates[5].market_cap, 17.6e6);

    // Scenarios for the last company
    assert_eq!(input.candidates[5].scenarios.len(), 3);
    assert_eq!(
        input.candidates[5].scenarios[0].thesis,
        "They don't manage to liquidate and just lose all the money"
    );
    assert_eq!(input.candidates[5].scenarios[0].intrinsic_value, 0.0);
    assert_eq!(input.candidates[5].scenarios[0].probability, 0.05);

    assert_eq!(
        input.candidates[5].scenarios[1].thesis,
        "They liquidate without realizing assets in escrow account, assuming significant quarterly \
        cash loss until liquidation\n"
    );
    assert_eq!(input.candidates[5].scenarios[1].intrinsic_value, 10.0e6);
    assert_eq!(input.candidates[5].scenarios[1].probability, 0.25);

    assert_eq!(
        input.candidates[5].scenarios[2].thesis,
        "They liquidate everything, assuming reasonable cash loss until liquidation"
    );
    assert_eq!(input.candidates[5].scenarios[2].intrinsic_value, 25.0e6);
    assert_eq!(input.candidates[5].scenarios[2].probability, 0.7);

    // Assert that there are no validation issues
    let logger = create_test_logger();
    let validation_errors: Vec<ValidationResult> = validate(&input, &logger);
    assert_eq!(validation_errors, vec![]);
}

#[test]
fn test_allocate_with_validation_problems() {
    // Change the probability of the first scenario from 0.05 to 0.03
    let mut input: AllocationInput = serde_yaml::from_str(&load_test_input_file()).unwrap();
    input.candidates[0].scenarios[0].probability = 0.03;

    let logger = create_test_logger();
    let allocation_response: AllocationResponse = allocate(input, &logger).unwrap().0;

    assert!(allocation_response.error.is_none());
    assert!(allocation_response.result.is_none());

    assert_eq!(
        allocation_response.validation_problems.unwrap(),
        vec![ValidationResult::PROBLEM(Problem {
            code: "probabilities-for-all-scenarios-do-not-sum-up-to-one".to_string(),
            message: "Probabilities of all scenarios for company A do not sum up to 1. Sum = 0.98."
                .to_string(),
            severity: Severity::ERROR,
        })],
    );
}

#[test]
fn test_allocate_with_no_candidates_after_filtering() {
    // Keep only two candidates and change the numbers such that one has negative expected return
    // and the other has no downside scenario
    let mut input: AllocationInput = serde_yaml::from_str(&load_test_input_file()).unwrap();
    input.candidates.pop();
    input.candidates.pop();
    input.candidates.pop();
    input.candidates.pop();

    // Swap probabilities of a worst-case scenario and base case scenario such that the expected
    // outcome is negative, for the first company
    input.candidates[0].scenarios[0].probability = 0.5;
    input.candidates[0].scenarios[2].probability = 0.05;

    // Remove first two (negative outcome) scenarios for the second company, and make the third one
    // have 100% probability
    input.candidates[1].scenarios.remove(0);
    input.candidates[1].scenarios.remove(0);
    input.candidates[1].scenarios[0].probability = 1.0;

    // Allocate and assert that we got an error
    let logger = create_test_logger();
    let allocation_response: AllocationResponse = allocate(input, &logger).unwrap().0;

    assert!(allocation_response.result.is_none());

    let validation_problems = allocation_response.validation_problems.unwrap();
    assert!(
        validation_problems.contains(&ValidationResult::PROBLEM(Problem {
            code: "negative-expected-return-for-a-company".to_string(),
            message:
                "Found negative expected return of -50.2% for A. This is not supported in the \
                current framework because we want to prohibit shorting."
                    .to_string(),
            severity: Severity::WARNING,
        }))
    );
    assert!(
        validation_problems.contains(&ValidationResult::PROBLEM(Problem {
            code: "company-with-no-downside-scenario".to_string(),
            message:
                "Company B doesn't have at least one downside scenario. This is not supported in \
                the current framework because the algorithm would try and tell you to put all your \
                money on this company."
                    .to_string(),
            severity: Severity::WARNING,
        }))
    );

    assert_eq!(
        allocation_response.error.unwrap(),
        Error {
            code: "no-valid-candidates-for-allocation".to_string(),
            message: "Found no valid candidates for allocation. Check your input.".to_string(),
        }
    );
}

/// Tests a case that doesn't converge since we have a company which has two scenarios that would
/// imply infinite leverage: One with extremely unlikely small downside and one with extremely
/// likely large upside.
#[test]
fn test_allocate_case_that_does_not_converge() {
    let mut input: AllocationInput = serde_yaml::from_str(&load_test_input_file()).unwrap();

    // Remove first scenario such that we're left with only two of them
    input.candidates[5].scenarios.remove(0);

    // Make the first scenario very unlikely with extremely small downside
    input.candidates[5].scenarios[0].probability = 1e-4;
    input.candidates[5].scenarios[0].intrinsic_value = 0.99 * input.candidates[5].market_cap;

    // Make the second scenario very likely with extremely large upside
    input.candidates[5].scenarios[1].probability = 1.0 - 1e-4;
    input.candidates[5].scenarios[1].intrinsic_value = 100.0 * input.candidates[5].market_cap;

    let logger = create_test_logger();
    let allocation_response: AllocationResponse = allocate(input, &logger).unwrap().0;

    assert_eq!(allocation_response.validation_problems.unwrap(), vec![]);
    assert!(allocation_response.result.is_none());

    let err = allocation_response.error.unwrap();

    assert_eq!(err.code, "did-not-find-a-single-viable-solution");
    assert!(err
        .message
        .contains("Did not manage to find a single viable numerical solution."));
    assert!(err
        .message
        .contains("Did not manage to find the numerical solution."));
}

/// Tests allocation for 6 candidate companies without constraints.
#[test]
fn test_allocate() {
    // Create candidates and validate them
    let logger = create_test_logger();
    let input: AllocationInput = serde_yaml::from_str(&load_test_input_file()).unwrap();
    let validation_errors: Vec<ValidationResult> = validate(&input, &logger);
    assert_eq!(validation_errors, vec![]);

    // Allocate
    let portfolio: AllocationResponse = allocate(input, &logger).unwrap().0;
    let tickers_and_fractions: Vec<TickerAndFraction> = portfolio.result.unwrap().allocations;

    // Debug convenience: To see the output, use create_logger(Info) instead of create_test_logger()
    info!(logger, "{:?}", tickers_and_fractions);

    assert_eq!(tickers_and_fractions[0].ticker, "A".to_string());
    assert_close!(
        0.1420462,
        tickers_and_fractions[0].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[1].ticker, "B".to_string());
    assert_close!(
        0.6407610,
        tickers_and_fractions[1].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[2].ticker, "C".to_string());
    assert_close!(
        0.5871123,
        tickers_and_fractions[2].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[3].ticker, "D".to_string());
    assert_close!(
        0.2316455,
        tickers_and_fractions[3].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[4].ticker, "E".to_string());
    assert_close!(
        0.3010064,
        tickers_and_fractions[4].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[5].ticker, "F".to_string());
    assert_close!(
        0.3498925,
        tickers_and_fractions[5].fraction,
        ASSERTION_TOLERANCE
    );
}

/// Tests allocation with all constraints, but only with three candidates because having more
/// candidates grows exponentially in complexity when we have constraints. Having just three of them
/// is enough for the integration test.
#[test]
fn test_allocate_with_constraints() {
    // Create candidates and validate them.
    let logger = create_logger(Info);

    let test_input_with_constraints = load_test_input_file()
        + &"
long_only: true
max_permanent_loss_of_capital:
    fraction_of_capital: 0.5
    probability_of_loss: 0.2
max_individual_allocation: 0.3
max_total_leverage_ratio: 0.0
    "
        .to_string();

    info!(logger, "\n{}", test_input_with_constraints);

    let mut input: AllocationInput = serde_yaml::from_str(&test_input_with_constraints).unwrap();
    input.candidates.pop(); // Remove F candidate
    input.candidates.pop(); // Remove E candidate
    input.candidates.remove(0); // Remove A candidate

    let validation_errors: Vec<ValidationResult> = validate(&input, &logger);
    assert_eq!(validation_errors, vec![]);

    // Allocate
    let portfolio: AllocationResponse = allocate(input, &logger).unwrap().0;
    let tickers_and_fractions: Vec<TickerAndFraction> = portfolio.result.unwrap().allocations;

    // Debug convenience: To see the output, use create_logger(Info) instead of create_test_logger()
    info!(logger, "{:?}", tickers_and_fractions);

    assert_eq!(tickers_and_fractions[0].ticker, "B".to_string());
    assert_close!(0.3, tickers_and_fractions[0].fraction, ASSERTION_TOLERANCE);

    assert_eq!(tickers_and_fractions[1].ticker, "C".to_string());
    assert_close!(0.3, tickers_and_fractions[1].fraction, ASSERTION_TOLERANCE);

    assert_eq!(tickers_and_fractions[2].ticker, "D".to_string());
    assert_close!(
        0.240576,
        tickers_and_fractions[2].fraction,
        ASSERTION_TOLERANCE
    );
}

/// Does the same allocation as in the [test_allocate] and asserts that the portfolio analysis
/// (statistics) are correct.
#[test]
fn test_analyze() {
    // Create candidates and validate them
    let logger = create_test_logger();
    let input: AllocationInput = serde_yaml::from_str(&load_test_input_file()).unwrap();
    let validation_errors: Vec<ValidationResult> = validate(&input, &logger);
    assert_eq!(validation_errors, vec![]);

    // Allocate and analyze
    let portfolio = KellyAllocator::new(&logger, MAX_ITER)
        .allocate(input.candidates)
        .unwrap();
    let analysis_response: AnalysisResponse = analyze(portfolio, &logger).unwrap().0;
    let analysis_result = analysis_response.result.unwrap();

    // Debug convenience: To see the output, use create_logger(Info) instead of create_test_logger()
    info!(logger, "{:?}", analysis_result);

    assert_close!(
        9.375e-5,
        analysis_result.worst_case_outcome.probability,
        ASSERTION_TOLERANCE
    );
    assert_close!(
        -1.608054,
        analysis_result.worst_case_outcome.portfolio_return,
        ASSERTION_TOLERANCE
    );
    assert_close!(
        -0.234950,
        analysis_result
            .worst_case_outcome
            .probability_weighted_return,
        ASSERTION_TOLERANCE
    );
    assert_close!(
        0.1896847,
        analysis_result.cumulative_probability_of_loss,
        ASSERTION_TOLERANCE
    );
    assert_close!(
        1.0971964,
        analysis_result.expected_return,
        ASSERTION_TOLERANCE
    );
}
