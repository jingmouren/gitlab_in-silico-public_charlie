use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};

use crate::scenario::Scenario;

/// Tolerance when validating that all probabilities across scenarios sum up to 1
const PROBABILITY_TOLERANCE: f64 = 1e-10;

/// A company with some basic information relevant for investment and a set of possible scenarios
#[derive(Serialize, Deserialize, Debug)]
pub struct Company {
    name: String,
    ticker: String,
    description: String,
    market_cap: f64,
    scenarios: Vec<Scenario>,
}

/// Two companies are considered equal if their ticker symbols are equal. This is done in order to
/// possibly handle in the future dually listed shares where some arbitrage may be present (i.e.
/// different market caps on different stock exchanges, for the same business).
impl PartialEq<Self> for Company {
    fn eq(&self, other: &Self) -> bool {
        self.ticker == other.ticker
    }
}

impl Eq for Company {}

/// Hash key based on the ticker symbol
impl Hash for Company {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.ticker.hash(hasher)
    }
}

impl Company {
    /// Does all validations. Used after construction
    pub fn validate(&self) {
        self.validate_at_least_one_scenario();
        self.validate_all_scenarios_unique();
        self.validate_probabilities_sum_up_to_one();
    }

    /// Panics if all scenarios are not unique
    /// TODO: Convert panics to recoverable errors that can be handled
    fn validate_all_scenarios_unique(&self) {
        if self.scenarios.len() != HashSet::<Scenario>::from_iter(self.scenarios.iter().cloned()).len() {
            panic!("Not all scenarios are unique (have a unique thesis). Check your input.")
        }
    }

    /// Panics if we don't have at least one scenario
    /// TODO: Convert panics to recoverable errors that can be handled
    fn validate_at_least_one_scenario(&self) {
        if self.scenarios.is_empty() {
            panic!(
                "No scenarios found for {name} with ticker {ticker}.",
                name = self.name, ticker = self.ticker
            )
        }
    }

    /// Panics if all probabilities across all scenarios don't sum up close to 1
    /// TODO: Convert panics to recoverable errors that can be handled
    fn validate_probabilities_sum_up_to_one(&self) {
        let sum: f64 = self.scenarios.iter().map(|scenario| scenario.probability).sum();
        if (sum - 1.0).abs() > PROBABILITY_TOLERANCE {
            panic!("Probabilities of all scenarios do not sum up to 1. Sum = {}.", sum)
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::hash_map::DefaultHasher;
    use super::*;

    #[test]
    fn test_probability_tolerance_doesnt_change() {
        assert_eq!(PROBABILITY_TOLERANCE, 1e-10)
    }

    #[test]
    fn test_company_serialization() {
        let test_company: Company = Company {
            name: "Some Company".to_string(),
            ticker: "SC".to_string(),
            description: "Some business that's pretty interesting.".to_string(),
            market_cap: 5e5,
            scenarios: vec![
                Scenario {
                    thesis: "Worst case liquidation value".to_string(),
                    intrinsic_value: 1e6,
                    probability: 0.6,
                },
                Scenario {
                    thesis: "Base case liquidation value".to_string(),
                    intrinsic_value: 2e6,
                    probability: 0.4,
                },
            ],
        };
        let test_str = serde_yaml::to_string(&test_company).unwrap();

        assert_eq!(test_str, "name: Some Company\nticker: SC\ndescription: Some business that's pretty interesting.\nmarket_cap: 500000.0\nscenarios:\n- thesis: Worst case liquidation value\n  intrinsic_value: 1000000.0\n  probability: 0.6\n- thesis: Base case liquidation value\n  intrinsic_value: 2000000.0\n  probability: 0.4\n");
    }

    #[test]
    fn test_company_deserialization() {
        let test_yaml: &str = "
            name: Some company
            ticker: SC
            description: Some business that's pretty interesting.
            market_cap: 5e5
            scenarios:
              - thesis: Worst case liquidation value
                intrinsic_value: 1e6
                probability: 0.6
              - thesis:  Base case liquidation value
                intrinsic_value: 2e6
                probability: 0.4
        ";

        let test_company: Company = serde_yaml::from_str(&test_yaml).unwrap();

        assert_eq!(test_company.name, "Some company");
        assert_eq!(test_company.ticker, "SC");
        assert_eq!(test_company.description, "Some business that's pretty interesting.");
        assert_eq!(test_company.market_cap, 5e5);

        assert_eq!(test_company.scenarios[0].thesis, "Worst case liquidation value");
        assert_eq!(test_company.scenarios[0].intrinsic_value, 1e6);
        assert_eq!(test_company.scenarios[0].probability, 0.6);

        assert_eq!(test_company.scenarios[1].thesis, "Base case liquidation value");
        assert_eq!(test_company.scenarios[1].intrinsic_value, 2e6);
        assert_eq!(test_company.scenarios[1].probability, 0.4);
    }

    #[test]
    #[should_panic(expected = "No scenarios found for Some Company with ticker SC.")]
    fn test_having_non_unique_scenarios_panics() {
        let test_company: Company = Company {
            name: "Some Company".to_string(),
            ticker: "SC".to_string(),
            description: "Some business that's pretty interesting.".to_string(),
            market_cap: 5e5,
            scenarios: vec![],
        };

        test_company.validate();
    }

    #[test]
    #[should_panic(expected = "Not all scenarios are unique (have a unique thesis).")]
    fn test_having_no_scenarios_panics() {
        let test_company: Company = Company {
            name: "Some Company".to_string(),
            ticker: "SC".to_string(),
            description: "Some business that's pretty interesting.".to_string(),
            market_cap: 5e5,
            scenarios: vec![
                Scenario {
                    thesis: "Same thesis as the other one.".to_string(),
                    intrinsic_value: 1e6,
                    probability: 0.6,
                },
                Scenario {
                    thesis: "Same thesis as the other one.".to_string(),
                    intrinsic_value: 2e6,
                    probability: 0.4,
                },
            ],
        };

        test_company.validate();
    }

    #[test]
    #[should_panic(expected = "Probabilities of all scenarios do not sum up to 1. Sum = 0.8.")]
    fn test_probabilities_not_summing_up_to_one_panics() {
        let test_company: Company = Company {
            name: "Some Company".to_string(),
            ticker: "SC".to_string(),
            description: "Some business that's pretty interesting.".to_string(),
            market_cap: 5e5,
            scenarios: vec![
                Scenario {
                    thesis: "Worst case liquidation value.".to_string(),
                    intrinsic_value: 1e6,
                    probability: 0.5,
                },
                Scenario {
                    thesis: "Base case liquidation value.".to_string(),
                    intrinsic_value: 2e6,
                    probability: 0.3,
                },
            ],
        };

        test_company.validate();
    }

    #[test]
    fn two_companies_with_same_ticker_are_equal_irrespective_of_other_fields() {
        let test_company_1 = Company {
            name: "Some fancy name 1".to_string(),
            ticker: "SFN".to_string(),
            description: "A description".to_string(),
            market_cap: 1e7,
            scenarios: vec![],
        };
        let test_company_2 = Company {
            name: "Some fancy name 2".to_string(),
            ticker: "SFN".to_string(),
            description: "A different description".to_string(),
            market_cap: 1e7,
            scenarios: vec![],
        };

        assert_eq!(test_company_1, test_company_2)
    }

    #[test]
    fn two_companies_with_same_ticker_have_equal_hash_irrespective_of_other_fields() {
        let test_company_1 = Company {
            name: "Some fancy name 1".to_string(),
            ticker: "SFN".to_string(),
            description: "A description".to_string(),
            market_cap: 1e7,
            scenarios: vec![],
        };
        let test_company_2 = Company {
            name: "Some fancy name 2".to_string(),
            ticker: "SFN".to_string(),
            description: "A different description".to_string(),
            market_cap: 1e7,
            scenarios: vec![],
        };

        let mut hasher = DefaultHasher::new();
        assert_eq!(test_company_1.hash(&mut hasher), test_company_2.hash(&mut hasher));
    }
}