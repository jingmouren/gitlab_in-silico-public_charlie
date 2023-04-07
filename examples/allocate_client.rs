use portfolio::allocation::FRACTION_TOLERANCE;
use portfolio::env::create_logger;
use portfolio::model::portfolio::PortfolioCandidates;
use portfolio::model::responses::AllocationResponse;
use reqwest::StatusCode;
use slog::{info, Level};

/// Make assertion tolerance the same as the fraction tolerance (no point in more accuracy)
const ASSERTION_TOLERANCE: f64 = FRACTION_TOLERANCE;

const TEST_YAML: &str = "
          companies:
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

/// Calls allocate endpoint on the localhost:8000 and asserts the results
fn main() {
    let logger = create_logger(Level::Info);

    // Create candidates and post
    info!(logger, "Preparing to post candidates to allocate endpoint.");
    let candidates: PortfolioCandidates = serde_yaml::from_str(&TEST_YAML.to_string()).unwrap();

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("http://localhost:8000/allocate")
        .json(&candidates)
        .send()
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let allocation_result = response.json::<AllocationResponse>().unwrap();
    info!(
        logger,
        "Post successful, allocation response is: {:?}", allocation_result
    );

    // Assert that the response is as expected
    info!(
        logger,
        "Asserting that we didn't hit run-time errors or validation problems."
    );
    assert_eq!(allocation_result.error, None);
    assert_eq!(allocation_result.validation_problems, Some(vec![]));

    // Assert allocation results
    info!(logger, "Asserting allocation results.");
    let result = allocation_result.result.unwrap();
    let tickers_and_fractions = result.allocations;

    assert_eq!(tickers_and_fractions[0].ticker, "D".to_string());
    assert!(
        (tickers_and_fractions[0].fraction - 0.2337).abs() < ASSERTION_TOLERANCE,
        "Expected close to 0.2337, got {}",
        tickers_and_fractions[0].fraction
    );

    assert_eq!(tickers_and_fractions[1].ticker, "E".to_string());
    assert!(
        (tickers_and_fractions[1].fraction - 0.3847).abs() < ASSERTION_TOLERANCE,
        "Expected close to 0.3847, got {}",
        tickers_and_fractions[1].fraction
    );

    assert_eq!(tickers_and_fractions[2].ticker, "F".to_string());
    assert!(
        (tickers_and_fractions[2].fraction - 0.3816).abs() < ASSERTION_TOLERANCE,
        "Expected close to 0.3816, got {}",
        tickers_and_fractions[2].fraction
    );

    // Assert analysis result
    info!(logger, "Asserting analysis results.");
    let analysis_result = result.analysis;

    assert!(
        (analysis_result.worst_case_outcome.probability - 0.00125).abs() < 1e-6,
        "Expected close to 0.00125, got {}",
        analysis_result.worst_case_outcome.probability
    );
    assert!(
        (analysis_result.worst_case_outcome.weighted_return + 0.8731).abs() < ASSERTION_TOLERANCE,
        "Expected close to -0.8731, got {}",
        analysis_result.worst_case_outcome.weighted_return
    );

    assert!(
        (analysis_result.cumulative_probability_of_loss - 0.38625).abs() < 1e-6,
        "Expected close to 0.38625, got {}",
        analysis_result.cumulative_probability_of_loss
    );

    assert!(
        (analysis_result.expected_return - 0.1429).abs() < ASSERTION_TOLERANCE,
        "Expected close to 0.1429, got {}",
        analysis_result.expected_return
    );

    info!(logger, "Done.");
}
