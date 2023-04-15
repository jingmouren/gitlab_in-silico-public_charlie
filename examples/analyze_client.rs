use charlie::allocation::FRACTION_TOLERANCE;
use charlie::env::create_logger;
use charlie::model::portfolio::Portfolio;
use charlie::model::responses::AnalysisResponse;
use charlie::utils::assert_close;
use reqwest::StatusCode;
use slog::{info, Level};

/// Make assertion tolerance the same as the fraction tolerance (no point in more accuracy)
const ASSERTION_TOLERANCE: f64 = FRACTION_TOLERANCE;

const TEST_YAML: &str = "
          companies:
            - company:
                description: Business D
                market_cap: 608000000
                name: D
                scenarios:
                  - intrinsic_value: 330000000
                    probability: 0.5
                    thesis: >
                      Assumes depressed normalized earnings, significantly higher future
                      capital expenditures than in the past, inability to pass on the
                      increased costs to customers, and a multiple of 10x.
                  - intrinsic_value: 1000000000
                    probability: 0.5
                    thesis: >
                      Assumes that the last year earnings are representative of future
                      earnings, with 15x multiple.
                ticker: D
              fraction: 0.12277426228476018
            - company:
                description: Business E
                market_cap: 441000000000
                name: E
                scenarios:
                  - intrinsic_value: 0
                    probability: 0.05
                    thesis: Unexpected stuff happens
                  - intrinsic_value: 320000000000
                    probability: 0.5
                    thesis: >
                      They lose market share and normalized earnings power by 10% a year
                      for five years, after which someone is willing to pay 8x earnings.
                  - intrinsic_value: 800000000000
                    probability: 0.45
                    thesis: >
                      They keep growing at 5% a year for five years, after which someone
                      is willing to pay 12x earnings.
                ticker: E
              fraction: 0.16602626739809112
            - company:
                description: Business F
                market_cap: 17600000
                name: F
                scenarios:
                  - intrinsic_value: 0
                    probability: 0.05
                    thesis: They don't manage to liquidate and just lose all the money
                  - intrinsic_value: 10000000
                    probability: 0.25
                    thesis: >
                      They liquidate without realizing assets in escrow account, assuming
                      significant quarterly cash loss until liquidation
                  - intrinsic_value: 25000000
                    probability: 0.7
                    thesis: >-
                      They liquidate everything, assuming reasonable cash loss until
                      liquidation
                ticker: F
              fraction: 0.18558683992589134
        ";

/// Calls the analyze endpoint on the localhost:8000 and asserts the results
fn main() {
    let logger = create_logger(Level::Info);

    // Create analysis input and post
    info!(logger, "Preparing to post portfolio to analyze endpoint.");
    let portfolio: Portfolio = serde_yaml::from_str(&TEST_YAML.to_string()).unwrap();

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("http://localhost:8000/analyze")
        .json(&portfolio)
        .send()
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let analysis_response = response.json::<AnalysisResponse>().unwrap();
    info!(
        logger,
        "Post successful, analysis response is: {:?}", analysis_response
    );

    // Assert that the response is as expected
    info!(logger, "Asserting that we didn't hit run-time errors.");
    assert_eq!(analysis_response.error, None);

    // Assert analysis results
    info!(logger, "Asserting analysis results.");
    let analysis_result = analysis_response.result.unwrap();

    assert_close!(
        0.00125,
        analysis_result.worst_case_outcome.probability,
        1e-6
    );
    assert_close!(
        -0.4078,
        analysis_result.worst_case_outcome.weighted_return,
        ASSERTION_TOLERANCE
    );
    assert_close!(0.4425, analysis_result.cumulative_probability_of_loss, 1e-6);
    assert_close!(
        0.06556,
        analysis_result.expected_return,
        ASSERTION_TOLERANCE
    );

    info!(logger, "Done.");
}
