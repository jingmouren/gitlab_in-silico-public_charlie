use epictetus::allocation::FRACTION_TOLERANCE;
use epictetus::env::create_logger;
use epictetus::model::portfolio::PortfolioCandidates;
use epictetus::model::responses::AllocationResponse;
use epictetus::utils::assert_close;
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
    assert_close!(
        0.2337,
        tickers_and_fractions[0].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[1].ticker, "E".to_string());
    assert_close!(
        0.3847,
        tickers_and_fractions[1].fraction,
        ASSERTION_TOLERANCE
    );

    assert_eq!(tickers_and_fractions[2].ticker, "F".to_string());
    assert_close!(
        0.3816,
        tickers_and_fractions[2].fraction,
        ASSERTION_TOLERANCE
    );

    // Assert analysis result
    info!(logger, "Asserting analysis results.");
    let analysis_result = result.analysis;

    assert_close!(
        0.00125,
        analysis_result.worst_case_outcome.probability,
        1e-6
    );
    assert_close!(
        -0.8731,
        analysis_result.worst_case_outcome.weighted_return,
        ASSERTION_TOLERANCE
    );
    assert_close!(
        0.38625,
        analysis_result.cumulative_probability_of_loss,
        1e-6
    );
    assert_close!(0.1429, analysis_result.expected_return, ASSERTION_TOLERANCE);

    info!(logger, "Done.");
}
