use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use crate::model::scenario::Scenario;
use crate::validation::result::{Problem, Severity, ValidationResult};
use crate::validation::validate::Validate;

pub type Ticker = String;

/// Tolerance for comparing floats
pub(crate) const TOLERANCE: f64 = 1e-10;

/// A company with some basic information relevant for investment and a set of possible scenarios
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Company {
    pub name: String,
    pub ticker: Ticker,
    pub description: String,
    pub market_cap: f64,
    pub scenarios: Vec<Scenario>,
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

impl Validate for Company {
    fn validate(&self) -> HashSet<ValidationResult> {
        let mut validation_results: HashSet<ValidationResult> = HashSet::new();

        validation_results.insert(self.validate_at_least_one_scenario());
        validation_results.insert(self.validate_all_scenarios_unique());
        validation_results.insert(self.validate_probabilities_sum_up_to_one());
        validation_results.extend(self.validate_all_scenarios());

        validation_results
    }
}

impl Company {
    /// Validates that we have at least one scenario
    fn validate_at_least_one_scenario(&self) -> ValidationResult {
        return if self.scenarios.is_empty() {
            ValidationResult::PROBLEM(Problem {
                code: "no-scenarios-for-company",
                message: format!(
                    "No scenarios found for {} with ticker {}.",
                    self.name, self.ticker
                ),
                severity: Severity::ERROR,
            })
        } else {
            ValidationResult::OK
        };
    }

    /// Validates that all scenarios have a unique thesis
    fn validate_all_scenarios_unique(&self) -> ValidationResult {
        let n_unique_scenarios =
            HashSet::<Scenario>::from_iter(self.scenarios.iter().cloned()).len();

        return if self.scenarios.len() != n_unique_scenarios {
            ValidationResult::PROBLEM(Problem {
                code: "scenarios-are-not-unique",
                message: format!(
                    "Not all scenarios have a unique thesis for company {}. Check your input.",
                    self.name
                ),
                severity: Severity::ERROR,
            })
        } else {
            ValidationResult::OK
        };
    }

    /// Validates that all probabilities across all scenarios sum up close to 1
    fn validate_probabilities_sum_up_to_one(&self) -> ValidationResult {
        let sum: f64 = self
            .scenarios
            .iter()
            .map(|scenario| scenario.probability)
            .sum();

        return if (sum - 1.0).abs() > TOLERANCE {
            ValidationResult::PROBLEM(Problem {
                code: "probabilities-for-all-scenarios-do-not-sum-up-to-one",
                message: format!("Probabilities of all scenarios do not sum up to 1. Sum = {sum}."),
                severity: Severity::ERROR,
            })
        } else {
            ValidationResult::OK
        };
    }

    /// Validate all scenarios individually
    fn validate_all_scenarios(&self) -> HashSet<ValidationResult> {
        let mut validation_results: HashSet<ValidationResult> = HashSet::new();

        self.scenarios
            .iter()
            .for_each(|s| validation_results.extend(s.validate()));

        validation_results
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_probability_tolerance_doesnt_change() {
        assert_eq!(TOLERANCE, 1e-10)
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
              - thesis: Base case liquidation value
                intrinsic_value: 2e6
                probability: 0.4
        ";

        let test_company: Company = serde_yaml::from_str(&test_yaml).unwrap();

        assert_eq!(test_company.name, "Some company");
        assert_eq!(test_company.ticker, "SC");
        assert_eq!(
            test_company.description,
            "Some business that's pretty interesting."
        );
        assert_eq!(test_company.market_cap, 5e5);

        assert_eq!(
            test_company.scenarios[0].thesis,
            "Worst case liquidation value"
        );
        assert_eq!(test_company.scenarios[0].intrinsic_value, 1e6);
        assert_eq!(test_company.scenarios[0].probability, 0.6);

        assert_eq!(
            test_company.scenarios[1].thesis,
            "Base case liquidation value"
        );
        assert_eq!(test_company.scenarios[1].intrinsic_value, 2e6);
        assert_eq!(test_company.scenarios[1].probability, 0.4);
    }

    #[test]
    fn test_validate_no_scenarios() {
        let test_company: Company = Company {
            name: "Some Company".to_string(),
            ticker: "SC".to_string(),
            description: "Some business that's pretty interesting.".to_string(),
            market_cap: 5e5,
            scenarios: vec![],
        };

        assert!(test_company
            .validate()
            .contains(&ValidationResult::PROBLEM(Problem {
                code: "no-scenarios-for-company",
                message: "No scenarios found for Some Company with ticker SC.".to_string(),
                severity: Severity::ERROR,
            })));
    }

    #[test]
    fn test_validate_non_unique_scenarios() {
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

        assert!(test_company
            .validate()
            .contains(&ValidationResult::PROBLEM(Problem {
            code: "scenarios-are-not-unique",
            message:
                "Not all scenarios have a unique thesis for company Some Company. Check your input."
                    .to_string(),
            severity: Severity::ERROR,
        })));
    }

    #[test]
    fn test_validate_probabilities_not_summing_up_to_one() {
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

        assert!(test_company
            .validate()
            .contains(&ValidationResult::PROBLEM(Problem {
                code: "probabilities-for-all-scenarios-do-not-sum-up-to-one",
                message: "Probabilities of all scenarios do not sum up to 1. Sum = 0.8."
                    .to_string(),
                severity: Severity::ERROR,
            })));
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
        assert_eq!(
            test_company_1.hash(&mut hasher),
            test_company_2.hash(&mut hasher)
        );
    }
}
