use portfolio::model::company::Company;
use portfolio::{allocate, create_candidates};

#[test]
fn test_create_candidates() {
    let test_yaml: &str = "
          - name: Company ABC
            ticker: ABC
            description: Interesting business 1
            market_cap: 1e6
            scenarios:
              - thesis: Worst case liquidation value
                intrinsic_value: 2e6
                probability: 0.6
              - thesis: Base case liquidation value
                intrinsic_value: 4e6
                probability: 0.4
          - name: Company XYZ
            ticker: XYZ
            description: Interesting business 2
            market_cap: 1e7
            scenarios:
              - thesis: Competition kills them faster then I think
                intrinsic_value: 5e6
                probability: 0.5
              - thesis: They manage to capitalize on current R&D
                intrinsic_value: 3e7
                probability: 0.5
        ";

    let candidates: Vec<Company> = create_candidates(&test_yaml.to_string());

    assert_eq!(candidates.len(), 2);

    // First company
    assert_eq!(candidates[0].name, "Company ABC");
    assert_eq!(candidates[0].ticker, "ABC");
    assert_eq!(candidates[0].description, "Interesting business 1");
    assert_eq!(candidates[0].market_cap, 1e6);

    // Scenarios for the first company
    assert_eq!(candidates[0].scenarios.len(), 2);
    assert_eq!(
        candidates[0].scenarios[0].thesis,
        "Worst case liquidation value"
    );
    assert_eq!(candidates[0].scenarios[0].intrinsic_value, 2e6);
    assert_eq!(candidates[0].scenarios[0].probability, 0.6);
    assert_eq!(
        candidates[0].scenarios[1].thesis,
        "Base case liquidation value"
    );
    assert_eq!(candidates[0].scenarios[1].intrinsic_value, 4e6);
    assert_eq!(candidates[0].scenarios[1].probability, 0.4);

    // Second company
    assert_eq!(candidates[1].name, "Company XYZ");
    assert_eq!(candidates[1].ticker, "XYZ");
    assert_eq!(candidates[1].description, "Interesting business 2");
    assert_eq!(candidates[1].market_cap, 1e7);

    // Scenarios for the second company
    assert_eq!(candidates[1].scenarios.len(), 2);
    assert_eq!(
        candidates[1].scenarios[0].thesis,
        "Competition kills them faster then I think"
    );
    assert_eq!(candidates[1].scenarios[0].intrinsic_value, 5e6);
    assert_eq!(candidates[1].scenarios[0].probability, 0.5);
    assert_eq!(
        candidates[1].scenarios[1].thesis,
        "They manage to capitalize on current R&D"
    );
    assert_eq!(candidates[1].scenarios[1].intrinsic_value, 3e7);
    assert_eq!(candidates[1].scenarios[1].probability, 0.5);
}

// TODO: Test allocate and analyze
