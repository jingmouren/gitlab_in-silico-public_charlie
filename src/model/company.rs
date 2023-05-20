use schemars::JsonSchema;
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
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
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
        validation_results.insert(self.validate_negative_expected_return());
        validation_results.insert(self.validate_no_downside_scenario());
        validation_results.extend(self.validate_all_scenarios());

        validation_results
    }
}

impl Company {
    /// Validates that we have at least one scenario
    fn validate_at_least_one_scenario(&self) -> ValidationResult {
        if self.scenarios.is_empty() {
            ValidationResult::PROBLEM(Problem {
                code: "no-scenarios-for-company".to_string(),
                message: format!(
                    "No scenarios found for {} with ticker {}.",
                    self.name, self.ticker
                ),
                severity: Severity::ERROR,
            })
        } else {
            ValidationResult::OK
        }
    }

    /// Validates that all scenarios have a unique thesis
    fn validate_all_scenarios_unique(&self) -> ValidationResult {
        let n_unique_scenarios =
            HashSet::<Scenario>::from_iter(self.scenarios.iter().cloned()).len();

        if self.scenarios.len() != n_unique_scenarios {
            ValidationResult::PROBLEM(Problem {
                code: "scenarios-are-not-unique".to_string(),
                message: format!(
                    "Not all scenarios have a unique thesis for company {}. Check your input.",
                    self.name
                ),
                severity: Severity::ERROR,
            })
        } else {
            ValidationResult::OK
        }
    }

    /// Validates that all probabilities across all scenarios sum up close to 1
    fn validate_probabilities_sum_up_to_one(&self) -> ValidationResult {
        let sum: f64 = self
            .scenarios
            .iter()
            .map(|scenario| scenario.probability)
            .sum();

        if (sum - 1.0).abs() > TOLERANCE {
            ValidationResult::PROBLEM(Problem {
                code: "probabilities-for-all-scenarios-do-not-sum-up-to-one".to_string(),
                message: format!("Probabilities of all scenarios for company {name} do not sum up to 1. Sum = {sum}.", name = self.name),
                severity: Severity::ERROR,
            })
        } else {
            ValidationResult::OK
        }
    }

    /// Return a validation warning if a company has a negative expected return. Within this
    /// framework (no shorting), this doesn't make sense.
    pub fn validate_negative_expected_return(&self) -> ValidationResult {
        let expected_return = self
            .scenarios
            .iter()
            .map(|s| s.probability * (s.intrinsic_value - self.market_cap) / self.market_cap)
            .sum::<f64>();

        if expected_return < 0.0 {
            ValidationResult::PROBLEM(Problem {
                code: "negative-expected-return-for-a-company".to_string(),
                message: format!(
                    "Found negative expected return of {:.1}% for {}. This is not supported in the \
                    current framework because we want to prohibit shorting.",
                    100.0 * expected_return,
                    self.ticker
                ),
                severity: Severity::WARNING,
            })
        } else {
            ValidationResult::OK
        }
    }

    /// Return a validation warning if a company doesn't have any downside scenario. This causes
    /// numerical failure because in this framework, the solution is to put an infinite bet on this
    /// company
    pub fn validate_no_downside_scenario(&self) -> ValidationResult {
        let has_no_downside = self.scenarios.iter().all(|s| {
            s.probability * (s.intrinsic_value - self.market_cap) / self.market_cap > -TOLERANCE
        });

        if has_no_downside {
            ValidationResult::PROBLEM(Problem {
                code: "company-with-no-downside-scenario".to_string(),
                message: format!(
                    "Company {} doesn't have at least one downside scenario. This is not supported \
                    in the current framework because the algorithm would try and tell you to put \
                    all your money on this company.",
                    self.ticker
                ),
                severity: Severity::WARNING,
            })
        } else {
            ValidationResult::OK
        }
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
                code: "no-scenarios-for-company".to_string(),
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
            code: "scenarios-are-not-unique".to_string(),
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
                code: "probabilities-for-all-scenarios-do-not-sum-up-to-one".to_string(),
                message: "Probabilities of all scenarios for company Some Company do not sum up to 1. Sum = 0.8."
                    .to_string(),
                severity: Severity::ERROR,
            })));
    }

    #[test]
    fn test_validate_validate_negative_expected_return() {
        let test_company: Company = Company {
            name: "Some Company".to_string(),
            ticker: "SC".to_string(),
            description: "Company with negative expected return.".to_string(),
            market_cap: 5e5,
            scenarios: vec![
                Scenario {
                    thesis: "Loss.".to_string(),
                    intrinsic_value: 1e5,
                    probability: 0.5,
                },
                Scenario {
                    thesis: "Zero return.".to_string(),
                    intrinsic_value: 5e5,
                    probability: 0.5,
                },
            ],
        };

        assert!(test_company
            .validate()
            .contains(&ValidationResult::PROBLEM(Problem {
                code: "negative-expected-return-for-a-company".to_string(),
                message:
                    "Found negative expected return of -40.0% for SC. This is not supported in the \
                    current framework because we want to prohibit shorting."
                        .to_string(),
                severity: Severity::WARNING,
            })));
    }

    #[test]
    fn test_validate_no_downside_scenario() {
        let test_company: Company = Company {
            name: "Some Company".to_string(),
            ticker: "SC".to_string(),
            description: "Company with no downside.".to_string(),
            market_cap: 5e5,
            scenarios: vec![
                Scenario {
                    thesis: "Breakeven.".to_string(),
                    intrinsic_value: 5e5,
                    probability: 0.5,
                },
                Scenario {
                    thesis: "Double.".to_string(),
                    intrinsic_value: 1e6,
                    probability: 0.5,
                },
            ],
        };

        assert!(test_company
            .validate()
            .contains(&ValidationResult::PROBLEM(Problem {
                code: "company-with-no-downside-scenario".to_string(),
                message: "Company SC doesn't have at least one downside scenario. This is not \
                    supported in the current framework because the algorithm would try and tell \
                    you to put all your money on this company."
                    .to_string(),
                severity: Severity::WARNING,
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
