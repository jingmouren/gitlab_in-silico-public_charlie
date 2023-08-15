use crate::constraints::constraint::{Constraint, InequalityConstraint};
use crate::model::portfolio::Portfolio;
use nalgebra::DVector;

/// [MaximumTotalLeverageConstraint] puts a constraint (upper bound) on the amount of leverage.
#[derive(Debug)]
pub struct MaximumTotalLeverageConstraint {
    /// Maximum leverage ratio, e.g. 0.0 means no leverage, while 1.0 means 100% leverage.
    max_leverage_ratio: f64,
}

impl MaximumTotalLeverageConstraint {
    /// Create a new [MaximumTotalLeverageConstraint] and fail if the provided leverage ratio is
    /// negative.
    pub fn new(max_leverage_ratio: f64) -> MaximumTotalLeverageConstraint {
        if max_leverage_ratio < 0.0 {
            panic!(
                "Maximum leverage ratio in a maximum total leverage constraint must be positive. \
                You provided {max_leverage_ratio}."
            )
        }

        MaximumTotalLeverageConstraint { max_leverage_ratio }
    }
}

impl InequalityConstraint for MaximumTotalLeverageConstraint {}

impl Constraint for MaximumTotalLeverageConstraint {
    fn d_constraint_d_fractions(&self, portfolio: &Portfolio) -> DVector<f64> {
        DVector::from_element(portfolio.companies.len(), 1.0)
    }

    fn function_value(&self, portfolio: &Portfolio, slack_variable: f64) -> f64 {
        self.d_constraint_d_fractions(portfolio)
            .iter()
            .enumerate()
            .map(|(c_i, dc_df)| dc_df * portfolio.companies[c_i].fraction)
            .sum::<f64>()
            + slack_variable
            - self.max_leverage_ratio
            - 1.0
    }
}
