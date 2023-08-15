use crate::constraints::constraint::{Constraint, InequalityConstraint};
use crate::model::portfolio::Portfolio;
use nalgebra::DVector;
use ordered_float::OrderedFloat;

/// [MaxCapitalLossConstraint] that puts an upper bound on the permanent loss of capital an investor
/// is comfortable with. It essentially limits the fractions such that the probability-weighted
/// worst-case scenario doesn't exceed the specified value.
#[derive(Debug)]
pub struct MaxCapitalLossConstraint {
    probability_times_fraction_of_capital_lost: f64,
}

impl MaxCapitalLossConstraint {
    /// Create a new [MaxCapitalLossConstraint] and check whether the probability weighted capital
    /// loss is negative. Note that by convention it must be negative because this represents a loss
    /// of capital.
    pub fn new(probability_times_capital_lost: f64) -> MaxCapitalLossConstraint {
        if probability_times_capital_lost > 0.0 {
            panic!(
                "Probability of worst-case scenario multiplied by the fraction of lost capital in \
                that scenario must be a negative number because it represents a loss. You \
                provided {probability_times_capital_lost}."
            )
        }

        if probability_times_capital_lost < -1.0 {
            panic!(
                "Probability of worst-case scenario multiplied by the fraction of lost capital in \
                that scenario cannot be lower than one because it would imply shorting or \
                probability higher than one. You provided {probability_times_capital_lost}."
            )
        }

        MaxCapitalLossConstraint {
            probability_times_fraction_of_capital_lost: probability_times_capital_lost,
        }
    }
}

impl InequalityConstraint for MaxCapitalLossConstraint {}

impl Constraint for MaxCapitalLossConstraint {
    fn d_constraint_d_fractions(&self, portfolio: &Portfolio) -> DVector<f64> {
        -DVector::from_vec(
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
            + self.probability_times_fraction_of_capital_lost
            + slack_variable
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic(
        expected = "Probability of worst-case scenario multiplied by the fraction of lost capital \
        in that scenario must be a negative number because it represents a loss. You provided 0.25."
    )]
    fn test_validate_positive_probability_times_fraction_of_capital_lost() {
        MaxCapitalLossConstraint::new(0.25);
    }

    #[test]
    #[should_panic(
        expected = "Probability of worst-case scenario multiplied by the fraction of lost capital \
        in that scenario cannot be lower than one because it would imply shorting or probability \
        higher than one. You provided -42."
    )]
    fn test_validate_probability_times_fraction_of_capital_lost_smaller_than_minus_one() {
        MaxCapitalLossConstraint::new(-42.0);
    }
}
