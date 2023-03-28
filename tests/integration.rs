use portfolio::allocation::{kelly_allocate, MAX_ITER};
use portfolio::env::create_logger;
use portfolio::model::portfolio::PortfolioCandidates;
use portfolio::model::responses::{AllocationResponse, AnalysisResponse, TickerAndFraction};
use portfolio::validation::result::ValidationResult;
use portfolio::{allocate, analyze, validate};
use slog::info;

const ASSERTION_TOLERANCE: f64 = 1e-6;

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
    let logger = create_logger();
    let validation_errors: Vec<ValidationResult> = validate(&candidates, &logger);
    assert_eq!(validation_errors, vec![]);
}

#[test]
fn test_allocate() {
    // Create candidates and validate them
    let logger = create_logger();
    let candidates: PortfolioCandidates = serde_yaml::from_str(&TEST_YAML.to_string()).unwrap();
    let validation_errors: Vec<ValidationResult> = validate(&candidates, &logger);
    assert_eq!(validation_errors, vec![]);

    // Allocate
    let portfolio: AllocationResponse = allocate(candidates, &logger).unwrap().0;
    let tickers_and_fractions: Vec<TickerAndFraction> = portfolio.result.unwrap().allocations;

    // Print out the result for convenience
    info!(logger, "{:?}", tickers_and_fractions);

    assert_eq!(tickers_and_fractions[0].ticker, "A".to_string());
    assert!(
        (tickers_and_fractions[0].fraction - 0.05066776266911893).abs() < ASSERTION_TOLERANCE,
        "Expected close to 0.05066776266911893, got {}",
        tickers_and_fractions[0].fraction
    );

    assert_eq!(tickers_and_fractions[1].ticker, "B".to_string());
    assert!(
        (tickers_and_fractions[1].fraction - 0.28662691955631936).abs() < ASSERTION_TOLERANCE,
        "Expected close to 0.28662691955631936, got {}",
        tickers_and_fractions[1].fraction
    );

    assert_eq!(tickers_and_fractions[2].ticker, "C".to_string());
    assert!(
        (tickers_and_fractions[2].fraction - 0.18831794816581915).abs() < ASSERTION_TOLERANCE,
        "Expected close to 0.18831794816581915, got {}",
        tickers_and_fractions[2].fraction
    );

    assert_eq!(tickers_and_fractions[3].ticker, "D".to_string());
    assert!(
        (tickers_and_fractions[3].fraction - 0.12277426228476018).abs() < ASSERTION_TOLERANCE,
        "Expected close to 0.12277426228476018, got {}",
        tickers_and_fractions[3].fraction
    );

    assert_eq!(tickers_and_fractions[4].ticker, "E".to_string());
    assert!(
        (tickers_and_fractions[4].fraction - 0.16602626739809112).abs() < ASSERTION_TOLERANCE,
        "Expected close to 0.16602626739809112, got {}",
        tickers_and_fractions[4].fraction
    );

    assert_eq!(tickers_and_fractions[5].ticker, "F".to_string());
    assert!(
        (tickers_and_fractions[5].fraction - 0.18558683992589134).abs() < ASSERTION_TOLERANCE,
        "Expected close to 0.18558683992589134, got {}",
        tickers_and_fractions[5].fraction
    );
}

#[test]
fn test_analyze() {
    // Create candidates and validate them
    let logger = create_logger();
    let candidates: PortfolioCandidates = serde_yaml::from_str(&TEST_YAML.to_string()).unwrap();
    let validation_errors: Vec<ValidationResult> = validate(&candidates, &logger);
    assert_eq!(validation_errors, vec![]);

    // Allocate and analyze
    let portfolio = kelly_allocate(candidates.companies, MAX_ITER, &logger).unwrap();
    let analysis_response: AnalysisResponse = analyze(portfolio, &logger).unwrap().0;
    let analysis_result = analysis_response.result.unwrap();

    // Print out the result for convenience
    info!(logger, "{:?}", analysis_result);

    assert!(
        (analysis_result.worst_case_outcome.probability - 3.125e-7).abs() < ASSERTION_TOLERANCE,
        "Expected close to 3.125e-7, got {}",
        analysis_result.worst_case_outcome.probability
    );
    assert!(
        (analysis_result.worst_case_outcome.weighted_return + 0.9333626536941269).abs()
            < ASSERTION_TOLERANCE,
        "Expected close to -0.9333626536941269, got {}",
        analysis_result.worst_case_outcome.weighted_return
    );

    assert!(
        (analysis_result.cumulative_probability_of_loss - 0.18054531249999986).abs()
            < ASSERTION_TOLERANCE,
        "Expected close to 0.18054531249999986, got {}",
        analysis_result.cumulative_probability_of_loss
    );

    assert!(
        (analysis_result.expected_return - 0.4274474630482845).abs() < ASSERTION_TOLERANCE,
        "Expected close to 0.4274474630482845, got {}",
        analysis_result.expected_return
    );
}
