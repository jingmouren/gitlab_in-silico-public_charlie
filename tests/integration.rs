use simple_logger::SimpleLogger;
use portfolio::model::company::Company;
use portfolio::{allocate, analyse, create_candidates, Portfolio};

const TEST_YAML: &str = "
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
fn test_create_candidates() {
    let candidates: Vec<Company> = create_candidates(&TEST_YAML.to_string());

    assert_eq!(candidates.len(), 6);

    // First company
    assert_eq!(candidates[0].name, "A");
    assert_eq!(candidates[0].ticker, "A");
    assert_eq!(candidates[0].description, "Business A");
    assert_eq!(candidates[0].market_cap, 238.0e9);

    // Scenarios for the first company
    assert_eq!(candidates[0].scenarios.len(), 4);
    assert_eq!(
        candidates[0].scenarios[0].thesis,
        "Unexpected stuff happens"
    );
    assert_eq!(candidates[0].scenarios[0].intrinsic_value, 0.0);
    assert_eq!(candidates[0].scenarios[0].probability, 0.05);

    assert_eq!(
        candidates[0].scenarios[1].thesis,
        "Core business keeps losing earnings power"
    );
    assert_eq!(candidates[0].scenarios[1].intrinsic_value, 170.0e9);
    assert_eq!(candidates[0].scenarios[1].probability, 0.3);

    assert_eq!(
        candidates[0].scenarios[2].thesis,
        "Business doesn't grow, earnings kept flat"
    );
    assert_eq!(candidates[0].scenarios[2].intrinsic_value, 270.0e9);
    assert_eq!(candidates[0].scenarios[2].probability, 0.5);

    assert_eq!(candidates[0].scenarios[3].thesis, "Earnings grow slightly");
    assert_eq!(candidates[0].scenarios[3].intrinsic_value, 360.0e9);
    assert_eq!(candidates[0].scenarios[3].probability, 0.15);

    // Last company
    assert_eq!(candidates[5].name, "F");
    assert_eq!(candidates[5].ticker, "F");
    assert_eq!(candidates[5].description, "Business F");
    assert_eq!(candidates[5].market_cap, 17.6e6);

    // Scenarios for the last company
    assert_eq!(candidates[5].scenarios.len(), 3);
    assert_eq!(
        candidates[5].scenarios[0].thesis,
        "They don't manage to liquidate and just lose all the money"
    );
    assert_eq!(candidates[5].scenarios[0].intrinsic_value, 0.0);
    assert_eq!(candidates[5].scenarios[0].probability, 0.05);

    assert_eq!(
        candidates[5].scenarios[1].thesis,
        "They liquidate without realizing assets in escrow account, assuming significant quarterly \
        cash loss until liquidation\n"
    );
    assert_eq!(candidates[5].scenarios[1].intrinsic_value, 10.0e6);
    assert_eq!(candidates[5].scenarios[1].probability, 0.25);

    assert_eq!(
        candidates[5].scenarios[2].thesis,
        "They liquidate everything, assuming reasonable cash loss until liquidation"
    );
    assert_eq!(candidates[5].scenarios[2].intrinsic_value, 25.0e6);
    assert_eq!(candidates[5].scenarios[2].probability, 0.7);
}

#[test]
fn test_allocate_and_analyze() {
    // Initialize logger
    SimpleLogger::new().init().unwrap();

    // TODO: Add assertions after refactoring the data classes and interfaces
    let candidates: Vec<Company> = create_candidates(&TEST_YAML.to_string());

    let portfolio: Portfolio = allocate(candidates);

    analyse(&portfolio);
}
