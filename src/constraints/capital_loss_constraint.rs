use crate::constraints::constraint::{Constraint, InequalityConstraint};
use crate::model::portfolio::Portfolio;
use nalgebra::DVector;
use ordered_float::OrderedFloat;

/// [CapitalLossConstraint] implements the [InequalityConstraint] interface that provides matrix
/// contributions to the Kelly allocation problem, both for inactive and active variants.
#[derive(Debug)]
pub struct CapitalLossConstraint {
    pub fraction_of_capital: f64,
    pub probability_of_loss: f64,
}

impl InequalityConstraint for CapitalLossConstraint {}

impl Constraint for CapitalLossConstraint {
    fn d_constraint_d_fractions(&self, portfolio: &Portfolio) -> DVector<f64> {
        DVector::from_vec(
            portfolio
                .companies
                .iter()
                .map(|p| {
                    p.company
                        .scenarios
                        .iter()
                        .map(|s| OrderedFloat(s.probability_weighted_return(p.company.market_cap)))
                        .min()
                        .unwrap_or_else(|| {
                            panic!(
                                "Did not manage to find worst case scenario for company {:?}",
                                p.company.ticker
                            )
                        })
                        .into_inner()
                })
                .collect(),
        )
    }

    fn function_value(&self, portfolio: &Portfolio, slack_variable: f64) -> f64 {
        self.d_constraint_d_fractions(portfolio)
            .iter()
            .enumerate()
            .map(|(c_i, dc_df)| dc_df * portfolio.companies[c_i].fraction)
            .sum::<f64>()
            - self.probability_of_loss * self.fraction_of_capital
            + slack_variable
    }

    fn is_satisfied(&self, portfolio: &Portfolio) -> bool {
        self.function_value(portfolio, 0.0) <= 0.0
    }
}
