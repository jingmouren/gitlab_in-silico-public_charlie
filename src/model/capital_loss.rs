use crate::model::company;
use crate::validation::result::{Problem, Severity, ValidationResult};
use crate::validation::validate::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Loss of capital is defined by two numbers: probability of the loss happening and the amount
/// lost. The data model is used in a constraint for modelling maximum allowable loss of capital.
/// Both numbers should be between 0 and 1.
/// This can be read as: "I'm ok losing [fraction] of capital with probability of [probability]."
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct CapitalLoss {
    pub fraction_of_capital: f64,
    pub probability_of_loss: f64,
}

impl Validate for CapitalLoss {
    /// Does all validations
    fn validate(&self) -> HashSet<ValidationResult> {
        let mut validation_results: HashSet<ValidationResult> = HashSet::new();

        validation_results.insert(self.validate_fraction_of_capital_bounds());
        validation_results.insert(self.validate_probability_of_loss_bounds());

        validation_results
    }
}

impl CapitalLoss {
    /// Validates that the probability of loss of capital is between 0 and 1
    fn validate_probability_of_loss_bounds(&self) -> ValidationResult {
        if self.probability_of_loss < company::TOLERANCE {
            return ValidationResult::PROBLEM(Problem {
                code: "probability-of-loss-of-capital-cannot-be-zero-or-negative".to_string(),
                message: format!(
                    "Zero or negative probability of loss is not allowed. Probability is {}.",
                    self.probability_of_loss
                ),
                severity: Severity::ERROR,
            });
        }

        if self.probability_of_loss > 1.0 {
            return ValidationResult::PROBLEM(Problem {
                code: "probability-of-loss-cannot-be-greater-than-one".to_string(),
                message: format!(
                    "Probability of loss greater than 1 is not allowed. Probability is {}.",
                    self.probability_of_loss
                ),
                severity: Severity::ERROR,
            });
        }

        ValidationResult::OK
    }

    /// Validates that the probability of loss of capital is between 0 and 1
    fn validate_fraction_of_capital_bounds(&self) -> ValidationResult {
        if self.fraction_of_capital < company::TOLERANCE {
            return ValidationResult::PROBLEM(Problem {
                code: "fraction-of-capital-cannot-be-zero-or-negative".to_string(),
                message: format!(
                    "Zero or negative fraction of capital lost is not allowed because this input \
                    represents the fraction of assets one is willing to lose under a worst-case \
                    scenario. Fraction is {}.",
                    self.fraction_of_capital
                ),
                severity: Severity::ERROR,
            });
        }

        if self.fraction_of_capital > 1.0 {
            return ValidationResult::PROBLEM(Problem {
                code: "fraction-of-capital-cannot-be-greater-than-one".to_string(),
                message: format!(
                    "Fraction of capital lost greater than 1 is not allowed because it would \
                    imply the use of leverage, which is not allowed for this constraint. \
                    Fraction is {}.",
                    self.fraction_of_capital
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

    #[test]
    fn test_validate_zero_probability_of_loss() {
        let test_capital_loss = CapitalLoss {
            probability_of_loss: 0.0,
            fraction_of_capital: 0.3,
        };
        assert!(test_capital_loss
            .validate()
            .contains(&ValidationResult::PROBLEM(Problem {
                code: "probability-of-loss-of-capital-cannot-be-zero-or-negative".to_string(),
                message: "Zero or negative probability of loss is not allowed. Probability is 0."
                    .to_string(),
                severity: Severity::ERROR,
            })));
    }

    #[test]
    fn test_validate_negative_probability_of_loss() {
        let test_capital_loss = CapitalLoss {
            probability_of_loss: -0.2,
            fraction_of_capital: 0.3,
        };
        assert!(test_capital_loss
            .validate()
            .contains(&ValidationResult::PROBLEM(Problem {
                code: "probability-of-loss-of-capital-cannot-be-zero-or-negative".to_string(),
                message:
                    "Zero or negative probability of loss is not allowed. Probability is -0.2."
                        .to_string(),
                severity: Severity::ERROR,
            })));
    }

    #[test]
    fn test_validate_probability_of_loss_greater_than_one() {
        let test_capital_loss = CapitalLoss {
            probability_of_loss: 1.2,
            fraction_of_capital: 0.3,
        };
        assert!(test_capital_loss
            .validate()
            .contains(&ValidationResult::PROBLEM(Problem {
                code: "probability-of-loss-cannot-be-greater-than-one".to_string(),
                message: "Probability of loss greater than 1 is not allowed. Probability is 1.2."
                    .to_string(),
                severity: Severity::ERROR,
            })));
    }

    #[test]
    fn test_validate_zero_fraction_of_capital() {
        let test_capital_loss = CapitalLoss {
            probability_of_loss: 0.5,
            fraction_of_capital: 0.0,
        };
        assert!(test_capital_loss
            .validate()
            .contains(&ValidationResult::PROBLEM(Problem {
                code: "fraction-of-capital-cannot-be-zero-or-negative".to_string(),
                message: "Zero or negative fraction of capital lost is not allowed because this \
                    input represents the fraction of assets one is willing to lose under a \
                    worst-case scenario. Fraction is 0."
                    .to_string(),
                severity: Severity::ERROR,
            })));
    }

    #[test]
    fn test_validate_negative_fraction_of_capital() {
        let test_capital_loss = CapitalLoss {
            probability_of_loss: 0.5,
            fraction_of_capital: -0.3,
        };
        assert!(test_capital_loss
            .validate()
            .contains(&ValidationResult::PROBLEM(Problem {
                code: "fraction-of-capital-cannot-be-zero-or-negative".to_string(),
                message: "Zero or negative fraction of capital lost is not allowed because this \
                    input represents the fraction of assets one is willing to lose under a \
                    worst-case scenario. Fraction is -0.3."
                    .to_string(),
                severity: Severity::ERROR,
            })));
    }

    #[test]
    fn test_validate_fraction_of_capital_greater_than_one() {
        let test_capital_loss = CapitalLoss {
            probability_of_loss: 0.5,
            fraction_of_capital: 2.2,
        };
        assert!(test_capital_loss
            .validate()
            .contains(&ValidationResult::PROBLEM(Problem {
                code: "fraction-of-capital-cannot-be-greater-than-one".to_string(),
                message: "Fraction of capital lost greater than 1 is not allowed because it would \
                    imply the use of leverage, which is not allowed for this constraint. \
                    Fraction is 2.2."
                    .to_string(),
                severity: Severity::ERROR,
            })));
    }
}
