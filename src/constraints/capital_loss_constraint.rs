use crate::constraints::constraint::{Constraint, InequalityConstraint};
use crate::model::portfolio::Portfolio;
use nalgebra::DVector;
use ordered_float::OrderedFloat;

/// [CapitalLossConstraint] that puts an upper bound on the permanent loss of capital the investor
/// is comfortable with. It essentially limits the fractions such that the probability-weighted
/// worst-case scenario doesn't exceed the specified value.
/// TODO: Fields can be represented into a single number.
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
}
