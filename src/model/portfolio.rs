use crate::model::capital_loss::CapitalLoss;
use crate::model::company::Company;
use crate::validation::result::{Problem, Severity, ValidationResult};
use crate::validation::validate::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Portfolio has a list of portfolio companies.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Portfolio {
    pub companies: Vec<PortfolioCompany>,
}

/// Portfolio company represents a company with an associated allocation fraction.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PortfolioCompany {
    pub company: Company,
    pub fraction: f64,
}

/// Allocation input consists of a list of candidate companies and additional constraints.
/// Note that the constraints are optional because the deserialization default for Option is None.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct AllocationInput {
    pub candidates: Vec<Company>,

    #[serde(default)]
    pub long_only: Option<bool>,

    #[serde(default)]
    pub max_permanent_loss_of_capital: Option<CapitalLoss>,

    #[serde(default)]
    pub max_individual_allocation: Option<f64>,

    #[serde(default)]
    pub max_total_leverage_ratio: Option<f64>,
}

impl Validate for AllocationInput {
    /// Validates the candidates and the constraints.
    fn validate(&self) -> HashSet<ValidationResult> {
        let mut validation_results: HashSet<ValidationResult> = HashSet::new();

        // Validate all candidates individually
        self.candidates
            .iter()
            .for_each(|c| validation_results.extend(c.validate()));

        // Validate maximum permanent loss of capital if specified
        if self.max_permanent_loss_of_capital.is_some() {
            validation_results.extend(
                self.max_permanent_loss_of_capital
                    .as_ref()
                    .unwrap()
                    .validate(),
            );
        }

        // If the maximum permanent loss of capital is set, we must have long-only constraint
        if self.max_permanent_loss_of_capital.is_some() && !self.long_only.unwrap_or(false) {
            validation_results.insert(ValidationResult::PROBLEM(Problem {
                code: "maximum-permanent-loss-constraint-works-only-with-long-only-constraint"
                    .to_string(),
                message: "Maximum permanent loss constraint works only with long-only constraint. \
                    Either remove the permanent loss constraint or use the long-only constraint."
                    .to_string(),
                severity: Severity::ERROR,
            }));
        }

        if self.max_individual_allocation.is_some() {
            let max_f = self.max_individual_allocation.unwrap();
            if max_f < 0.0 {
                validation_results.insert(ValidationResult::PROBLEM(Problem {
                    code: "maximum-individual-allocation-cannot-be-negative".to_string(),
                    message: format!(
                        "Maximum individual allocation cannot be negative. You provided {max_f}."
                    )
                    .to_string(),
                    severity: Severity::ERROR,
                }));
            }
        }

        if self.max_total_leverage_ratio.is_some() {
            let max_lr = self.max_total_leverage_ratio.unwrap();
            if max_lr < 0.0 {
                validation_results.insert(ValidationResult::PROBLEM(Problem {
                    code: "maximum-total-leverage-ratio-cannot-be-negative".to_string(),
                    message: format!(
                        "Maximum total leverage ratio cannot be negative. You provided {max_lr}."
                    )
                    .to_string(),
                    severity: Severity::ERROR,
                }));
            }
        }

        validation_results
    }
}
