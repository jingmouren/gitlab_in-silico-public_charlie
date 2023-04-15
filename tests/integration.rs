use charlie::allocation::{kelly_allocate, FRACTION_TOLERANCE, MAX_ITER};
use charlie::env::create_test_logger;
use charlie::model::errors::Error;
use charlie::model::portfolio::PortfolioCandidates;
use charlie::model::responses::{AllocationResponse, AnalysisResponse, TickerAndFraction};
use charlie::utils::assert_close;
use charlie::validation::result::{Problem, Severity, ValidationResult};
use charlie::{allocate, analyze, validate};
use slog::info;

/// Make assertion tolerance the same as the fraction tolerance (no point in more accuracy)
const ASSERTION_TOLERANCE: f64 = FRACTION_TOLERANCE;

const TEST_YAML: &str = "
          companies:
          - name: A
            ticker: A
            description: Business A
            market_cap: 238.0e9
            scenarios:
              - thesis: Unexpected stuff happens
                intrinsic_value: 0.0
                probability: 0.05
              - thesis: Core business keeps losing earnings power
                intrinsic_value: 170.0e9
                probability: 0.3
              - thesis: Business doesn't grow, earnings kept flat
                intrinsic_value: 270.0e9
                probability: 0.5
              - thesis: Earnings grow slightly
                intrinsic_value: 360.0e9
                probability: 0.15

          - name: B
            ticker: B
            description: Business B
            market_cap: 363.0e6
            scenarios:
              - thesis: Unexpected stuff happens
                intrinsic_value: 0.0
                probability: 0.05
              - thesis: Liquidation value
                intrinsic_value: 350.0e6
                probability: 0.5
              - thesis: Cycle moves upward and the market values the business correctly
                intrinsic_value: 900.0e6
                probability: 0.45

          - name: C
            ticker: C
            description: Business C
            market_cap: 35.3e6
            scenarios:
              - thesis: >
                    They don't manage to liquidate and it turns out that they're incompetent as
                    they were in the past
                intrinsic_value: 0.0
                probability: 0.1
              - thesis: They manage to liquidate at 25% of similar realized prices in the past
                intrinsic_value: 33.5e6
                probability: 0.5
              - thesis: They manage to liquidate at 50% of similar realized prices in the past
                intrinsic_value: 135.0e6
                probability: 0.4

          - name: D
            ticker: D
            description: Business D
            market_cap: 608.0e6
            scenarios:
              - thesis: >
                    Assumes depressed normalized earnings, significantly higher future capital
                    expenditures than in the past, inability to pass on the increased costs to
                    customers, and a multiple of 10x.
                intrinsic_value: 330.0e6
                probability: 0.5
              - thesis: >
                    Assumes that the last year earnings are representative of future earnings,
                    with 15x multiple.
                intrinsic_value: 1000.0e6
                probability: 0.5

          - name: E
            ticker: E
            description: Business E
            market_cap: 441.0e9
            scenarios:
              - thesis: Unexpected stuff happens
                intrinsic_value: 0.0
                probability: 0.05
              - thesis: >
                    They lose market share and normalized earnings power by 10% a year for five
                    years, after which someone is willing to pay 8x earnings.
                intrinsic_value: 320.0e9
                probability: 0.5
              - thesis: >
                    They keep growing at 5% a year for five years, after which someone is willing
                    to pay 12x earnings.
                intrinsic_value: 800.0e9
                probability: 0.45

          - name: F
            ticker: F
            description: Business F
            market_cap: 17.6e6
            scenarios:
              - thesis: They don't manage to liquidate and just lose all the money
                intrinsic_value: 0.0
                probability: 0.05
              - thesis: >
                    They liquidate without realizing assets in escrow account, assuming significant
                    quarterly cash loss until liquidation
                intrinsic_value: 10.0e6
                probability: 0.25
              - thesis: They liquidate everything, assuming reasonable cash loss until liquidation
                intrinsic_value: 25.0e6
                probability: 0.7
        ";

#[test]
fn test_create_candidates_and_validate() {
    let candidates: PortfolioCandidates = serde_yaml::from_str(&TEST_YAML.to_string()).unwrap();

    assert_eq!(candidates.companies.len(), 6);

    // First company
    assert_eq!(candidates.companies[0].name, "A");
    assert_eq!(candidates.companies[0].ticker, "A");
    assert_eq!(candidates.companies[0].description, "Business A");
    assert_eq!(candidates.companies[0].market_cap, 238.0e9);

    // Scenarios for the first company
    assert_eq!(candidates.companies[0].scenarios.len(), 4);
    assert_eq!(
        candidates.companies[0].scenarios[0].thesis,
        "Unexpected stuff happens"
    );
    assert_eq!(candidates.companies[0].scenarios[0].intrinsic_value, 0.0);
    assert_eq!(candidates.companies[0].scenarios[0].probability, 0.05);

    assert_eq!(
        candidates.companies[0].scenarios[1].thesis,
        "Core business keeps losing earnings power"
    );
    assert_eq!(
        candidates.companies[0].scenarios[1].intrinsic_value,
        170.0e9
    );
    assert_eq!(candidates.companies[0].scenarios[1].probability, 0.3);

    assert_eq!(
        candidates.companies[0].scenarios[2].thesis,
        "Business doesn't grow, earnings kept flat"
    );
    assert_eq!(
        candidates.companies[0].scenarios[2].intrinsic_value,
        270.0e9
    );
    assert_eq!(candidates.companies[0].scenarios[2].probability, 0.5);

    assert_eq!(
        candidates.companies[0].scenarios[3].thesis,
        "Earnings grow slightly"
    );
    assert_eq!(
        candidates.companies[0].scenarios[3].intrinsic_value,
        360.0e9
    );
    assert_eq!(candidates.companies[0].scenarios[3].probability, 0.15);

    // Last company
    assert_eq!(candidates.companies[5].name, "F");
    assert_eq!(candidates.companies[5].ticker, "F");
    assert_eq!(candidates.companies[5].description, "Business F");
    assert_eq!(candidates.companies[5].market_cap, 17.6e6);

    // Scenarios for the last company
    assert_eq!(candidates.companies[5].scenarios.len(), 3);
    assert_eq!(
        candidates.companies[5].scenarios[0].thesis,
        "They don't manage to liquidate and just lose all the money"
    );
    assert_eq!(candidates.companies[5].scenarios[0].intrinsic_value, 0.0);
    assert_eq!(candidates.companies[5].scenarios[0].probability, 0.05);

    assert_eq!(
        candidates.companies[5].scenarios[1].thesis,
        "They liquidate without realizing assets in escrow account, assuming significant quarterly \
        cash loss until liquidation\n"
    );
    assert_eq!(candidates.companies[5].scenarios[1].intrinsic_value, 10.0e6);
    assert_eq!(candidates.companies[5].scenarios[1].probability, 0.25);

    assert_eq!(
        candidates.companies[5].scenarios[2].thesis,
        "They liquidate everything, assuming reasonable cash loss until liquidation"
    );
    assert_eq!(candidates.companies[5].scenarios[2].intrinsic_value, 25.0e6);
    assert_eq!(candidates.companies[5].scenarios[2].probability, 0.7);

    // Assert that there are no validation issues
    let logger = create_test_logger();
    let validation_errors: Vec<ValidationResult> = validate(&candidates, &logger);
    assert_eq!(validation_errors, vec![]);
}

#[test]
fn test_allocate_with_validation_problems() {
    // Change the probability of the first scenario from 0.05 to 0.03
    let mut candidates: PortfolioCandidates = serde_yaml::from_str(&TEST_YAML.to_string()).unwrap();
    candidates.companies[0].scenarios[0].probability = 0.03;

    let logger = create_test_logger();
    let allocation_response: AllocationResponse = allocate(candidates, &logger).unwrap().0;

    assert!(allocation_response.error.is_none());
    assert!(allocation_response.result.is_none());

    assert_eq!(
        allocation_response.validation_problems.unwrap(),
        vec![ValidationResult::PROBLEM(Problem {
            code: "probabilities-for-all-scenarios-do-not-sum-up-to-one".to_string(),
            message: "Probabilities of all scenarios do not sum up to 1. Sum = 0.98.".to_string(),
            severity: Severity::ERROR,
        })],
    );
}

#[test]
fn test_allocate_with_no_candidates_after_filtering() {
    // Keep only two candidates and change the numbers such that one has negative expected return
    // and the other has no downside scenario
    let mut candidates: PortfolioCandidates = serde_yaml::from_str(&TEST_YAML.to_string()).unwrap();
    candidates.companies.pop();
    candidates.companies.pop();
    candidates.companies.pop();
    candidates.companies.pop();

    // Swap probabilities of a worst-case scenario and base case scenario such that the expected
    // outcome is negative, for the first company
    candidates.companies[0].scenarios[0].probability = 0.5;
    candidates.companies[0].scenarios[2].probability = 0.05;

    // Remove first two (negative outcome) scenarios for the second company, and make the third one
    // have 100% probability
    candidates.companies[1].scenarios.remove(0);
    candidates.companies[1].scenarios.remove(0);
    candidates.companies[1].scenarios[0].probability = 1.0;

    // Allocate and assert that we got an error
    let logger = create_test_logger();
    let allocation_response: AllocationResponse = allocate(candidates, &logger).unwrap().0;

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

#[test]
fn test_allocate_case_that_does_not_converge() {
    let mut candidates: PortfolioCandidates = serde_yaml::from_str(&TEST_YAML.to_string()).unwrap();

    // Remove first scenario such that we're left with only two of them
    candidates.companies[5].scenarios.remove(0);

    // Make the first scenario very unlikely with extremely small downside
    candidates.companies[5].scenarios[0].probability = 1e-3;
    candidates.companies[5].scenarios[0].intrinsic_value =
        0.99 * candidates.companies[5].market_cap;

    // Make the second scenario very likely with extremely large upside
    candidates.companies[5].scenarios[1].probability = 1.0 - 1e-3;
    candidates.companies[5].scenarios[1].intrinsic_value =
        100.0 * candidates.companies[5].market_cap;

    let logger = create_test_logger();
    let allocation_response: AllocationResponse = allocate(candidates, &logger).unwrap().0;
    println!("{:?}", allocation_response);

    assert_eq!(allocation_response.validation_problems.unwrap(), vec![]);
    assert!(allocation_response.result.is_none());

    assert_eq!(
        allocation_response.error.unwrap(),
        Error {
            code: "nonlinear-loop-didnt-converge".to_string(),
            message:
                "Did not manage to find the numerical solution. This may happen if the input data \
                would suggest a very strong bias towards a single/few investments. Check your \
                input."
                    .to_string(),
        },
    );
}

#[test]
fn test_allocate() {
    // Create candidates and validate them
    let logger = create_test_logger();
    let candidates: PortfolioCandidates = serde_yaml::from_str(&TEST_YAML.to_string()).unwrap();
    let validation_errors: Vec<ValidationResult> = validate(&candidates, &logger);
    assert_eq!(validation_errors, vec![]);

    // Allocate
    let portfolio: AllocationResponse = allocate(candidates, &logger).unwrap().0;
    let tickers_and_fractions: Vec<TickerAndFraction> = portfolio.result.unwrap().allocations;

    // Print out the result for convenience
    info!(logger, "{:?}", tickers_and_fractions);

    assert_eq!(tickers_and_fractions[0].ticker, "A".to_string());
    assert_close!(
        0.066,
        tickers_and_fractions[0].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[1].ticker, "B".to_string());
    assert_close!(
        0.282,
        tickers_and_fractions[1].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[2].ticker, "C".to_string());
    assert_close!(
        0.259,
        tickers_and_fractions[2].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[3].ticker, "D".to_string());
    assert_close!(
        0.102,
        tickers_and_fractions[3].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[4].ticker, "E".to_string());
    assert_close!(
        0.136,
        tickers_and_fractions[4].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[5].ticker, "F".to_string());
    assert_close!(
        0.154,
        tickers_and_fractions[5].fraction,
        ASSERTION_TOLERANCE
    );
}

#[test]
fn test_analyze() {
    // Create candidates and validate them
    let logger = create_test_logger();
    let candidates: PortfolioCandidates = serde_yaml::from_str(&TEST_YAML.to_string()).unwrap();
    let validation_errors: Vec<ValidationResult> = validate(&candidates, &logger);
    assert_eq!(validation_errors, vec![]);

    // Allocate and analyze
    let portfolio = kelly_allocate(candidates.companies, MAX_ITER, &logger).unwrap();
    let analysis_response: AnalysisResponse = analyze(portfolio, &logger).unwrap().0;
    let analysis_result = analysis_response.result.unwrap();

    // Print out the result for convenience
    info!(logger, "{:?}", analysis_result);

    assert_close!(
        3.125e-7,
        analysis_result.worst_case_outcome.probability,
        ASSERTION_TOLERANCE
    );
    assert_close!(
        -0.944,
        analysis_result.worst_case_outcome.weighted_return,
        ASSERTION_TOLERANCE
    );
    assert_close!(
        0.19,
        analysis_result.cumulative_probability_of_loss,
        ASSERTION_TOLERANCE
    );
    assert_close!(0.484, analysis_result.expected_return, ASSERTION_TOLERANCE);
}
