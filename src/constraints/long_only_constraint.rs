use crate::constraints::constraint::{Constraint, InequalityConstraint};
use crate::model::portfolio::Portfolio;
use nalgebra::DVector;

/// [LongOnlyConstraint] puts a constraint that essentially disallows shorting. It limits the
/// fractions such that they are always non-negative.
#[derive(Debug)]
pub struct LongOnlyConstraint {
    /// Index representing the company (i.e. the fraction) it constrains.
    fraction_index: usize,
}

impl LongOnlyConstraint {
    /// Create a new [LongOnlyConstraint] and perform a basic sanity check.
    pub fn new(fraction_index: usize, n_companies: usize) -> LongOnlyConstraint {
        if fraction_index > n_companies - 1 {
            panic!(
                "You have {n_companies} companies, but provided company ID {fraction_index}. \
            The company (fraction) ID must be smaller than the number of companies."
            )
        }

        LongOnlyConstraint { fraction_index }
    }
}

impl InequalityConstraint for LongOnlyConstraint {}

impl Constraint for LongOnlyConstraint {
    fn d_constraint_d_fractions(&self, portfolio: &Portfolio) -> DVector<f64> {
        let mut derivative: DVector<f64> = DVector::zeros(portfolio.companies.len());
        derivative[self.fraction_index] = -1.0;
        derivative
    }

    fn function_value(&self, portfolio: &Portfolio, slack_variable: f64) -> f64 {
        -portfolio.companies[self.fraction_index].fraction + slack_variable
    }
}
