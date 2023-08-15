use crate::constraints::constraint::{Constraint, InequalityConstraint};
use crate::model::portfolio::Portfolio;
use nalgebra::DVector;

/// [MaximumIndividualAllocationConstraint] puts a constraint (upper bound) on the amount of assets
/// to put into a single company.
#[derive(Debug)]
pub struct MaximumIndividualAllocationConstraint {
    /// Index representing the company (i.e. the fraction) it constrains.
    fraction_index: usize,

    /// Maximum allocation fraction for this company.
    max_allocation_fraction: f64,
}

impl MaximumIndividualAllocationConstraint {
    /// Create a new [MaximumIndividualAllocationConstraint] and perform some sanity checks.
    pub fn new(
        fraction_index: usize,
        max_allocation_fraction: f64,
        n_companies: usize,
    ) -> MaximumIndividualAllocationConstraint {
        if max_allocation_fraction < 0.0 {
            panic!(
                "Maximum allocation fraction must be positive. You provided {max_allocation_fraction}."
            )
        }

        if fraction_index > n_companies - 1 {
            panic!(
                "You have {n_companies} companies, but provided company ID {fraction_index}. \
            The company (fraction) ID must be smaller than the number of companies."
            )
        }

        MaximumIndividualAllocationConstraint {
            fraction_index,
            max_allocation_fraction,
        }
    }
}

impl InequalityConstraint for MaximumIndividualAllocationConstraint {}

impl Constraint for MaximumIndividualAllocationConstraint {
    fn d_constraint_d_fractions(&self, portfolio: &Portfolio) -> DVector<f64> {
        let mut derivative: DVector<f64> = DVector::zeros(portfolio.companies.len());
        derivative[self.fraction_index] = 1.0;
        derivative
    }

    fn function_value(&self, portfolio: &Portfolio, slack_variable: f64) -> f64 {
        portfolio.companies[self.fraction_index].fraction + slack_variable
            - self.max_allocation_fraction
    }
}
