use crate::validation::result::{Problem, Severity, ValidationResult};
use crate::validation::validate::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// A scenario is represented by an investment thesis, which can be boiled down to the expected
/// intrinsic value and the estimated probability that this scenario will play out in the future.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Scenario {
    pub thesis: String,
    pub intrinsic_value: f64,
    pub probability: f64,
}

/// Two scenarios are considered equal if their theses are equal, irrespective of the numbers.
impl PartialEq<Self> for Scenario {
    fn eq(&self, other: &Self) -> bool {
        self.thesis == other.thesis
    }
}

impl Eq for Scenario {}

/// Hash key based on the thesis.
impl Hash for Scenario {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.thesis.hash(hasher)
    }
}

impl Validate for Scenario {
    /// Does all validations.
    fn validate(&self) -> HashSet<ValidationResult> {
        HashSet::from([self.validate_probability_bounds()])
    }
}

impl Scenario {
    /// Calculates the return of this scenario given the market cap.
    pub fn scenario_return(&self, market_cap: f64) -> f64 {
        (self.intrinsic_value - market_cap) / market_cap
    }

    /// Calculates the probability weighted return for this scenario given the market cap.
    pub fn probability_weighted_return(&self, market_cap: f64) -> f64 {
        self.probability * self.scenario_return(market_cap)
    }

    /// Validates that all the probabilities are between 0 and 1.
    fn validate_probability_bounds(&self) -> ValidationResult {
        if self.probability < 0.0 {
            return ValidationResult::PROBLEM(Problem {
                code: "negative-probability-for-scenario".to_string(),
                message: format!(
                    "Negative probability is not allowed. Probability: {}",
                    self.probability
                ),
                severity: Severity::ERROR,
            });
        }

        if self.probability > 1.0 {
            return ValidationResult::PROBLEM(Problem {
                code: "probability-for-scenario-greater-than-one".to_string(),
                message: format!(
                    "Probability greater than 1 is not allowed. Probability: {}",
                    self.probability
                ),
                severity: Severity::ERROR,
            });
        }

        ValidationResult::OK
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_close;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_scenario_serialization() {
        let test_scenario: Scenario = Scenario {
            thesis: "Liquidation value".to_string(),
            intrinsic_value: 1e6,
            probability: 0.6,
        };
        let test_str = serde_yaml::to_string(&test_scenario).unwrap();

        assert_eq!(
            test_str,
            "thesis: Liquidation value\nintrinsic_value: 1000000.0\nprobability: 0.6\n"
        );
    }

    #[test]
    fn test_scenario_deserialization() {
        let test_yaml: &str = "
            thesis: Liquidation value
            intrinsic_value: 1e6
            probability: 0.6
        ";

        let test_scenario: Scenario = serde_yaml::from_str(&test_yaml).unwrap();

        assert_eq!(test_scenario.thesis, "Liquidation value");
        assert_eq!(test_scenario.intrinsic_value, 1e6);
        assert_eq!(test_scenario.probability, 0.6);
    }

    #[test]
    fn test_scenario_return() {
        let test_scenario = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1e6,
            probability: 0.2,
        };
        assert_close!(test_scenario.scenario_return(2e6), -0.5, 1e-10);
    }

    #[test]
    fn test_probability_weighted_return() {
        let test_scenario = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1e6,
            probability: 0.2,
        };
        assert_close!(test_scenario.probability_weighted_return(2e6), -0.1, 1e-10);
    }

    #[test]
    fn test_validate_negative_probability() {
        let test_scenario = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1e10,
            probability: -0.2,
        };
        assert_eq!(
            test_scenario.validate(),
            HashSet::from([ValidationResult::PROBLEM(Problem {
                code: "negative-probability-for-scenario".to_string(),
                message: "Negative probability is not allowed. Probability: -0.2".to_string(),
                severity: Severity::ERROR,
            })])
        );
    }

    #[test]
    fn test_validate_probability_greater_than_one() {
        let test_scenario = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1e10,
            probability: 1.2,
        };
        assert_eq!(
            test_scenario.validate(),
            HashSet::from([ValidationResult::PROBLEM(Problem {
                code: "probability-for-scenario-greater-than-one".to_string(),
                message: "Probability greater than 1 is not allowed. Probability: 1.2".to_string(),
                severity: Severity::ERROR,
            })])
        );
    }

    #[test]
    fn two_scenarios_with_same_thesis_are_equal_irrespective_of_different_intrinsic_value() {
        let test_scenario_1 = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1.2e7,
            probability: 0.3,
        };
        let test_scenario_2 = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1.2e8,
            probability: 0.4,
        };
        assert_eq!(test_scenario_1, test_scenario_2)
    }

    #[test]
    fn two_scenarios_with_same_thesis_have_equal_hash_irrespective_of_different_intrinsic_value() {
        let test_scenario_1 = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1.2e7,
            probability: 0.3,
        };
        let test_scenario_2 = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1.2e8,
            probability: 0.4,
        };

        let mut hasher = DefaultHasher::new();
        assert_eq!(
            test_scenario_1.hash(&mut hasher),
            test_scenario_2.hash(&mut hasher)
        );
    }
}
