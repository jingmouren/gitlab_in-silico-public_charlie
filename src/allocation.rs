use crate::analysis::Outcome;
use crate::model::company::Company;
use nalgebra::DMatrix;
use num_traits::pow::Pow;

/// Calculates the Jacobian given all outcomes, companies and their fractions
pub fn jacobian(
    outcomes: &[Outcome],
    candidates_and_fractions: Vec<(&Company, f64)>,
) -> DMatrix<f64> {
    let n_companies = candidates_and_fractions.len();
    let mut jacobian = DMatrix::zeros(n_companies, n_companies);

    for row_index in 0..n_companies {
        for column_index in 0..n_companies {
            let row_company: &Company = candidates_and_fractions[row_index].0;
            let column_company: &Company = candidates_and_fractions[column_index].0;

            jacobian[(row_index, column_index)] = -outcomes
                .iter()
                .map(|o| {
                    o.probability
                        * o.company_returns[&row_company.ticker]
                        * o.company_returns[&column_company.ticker]
                        * (1.0
                            + candidates_and_fractions
                                .iter()
                                .map(|(c, f)| f * o.company_returns[&c.ticker])
                                .sum::<f64>())
                        .pow(-2)
                })
                .sum::<f64>()
        }
    }

    jacobian
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::company;
    use crate::model::company::Company;
    use crate::model::scenario::Scenario;
    use std::collections::HashMap;

    const ASSERTION_TOLERANCE: f64 = 1e-6;

    #[test]
    fn test_jacobian() {
        let candidates: Vec<Company> = vec![
            Company {
                name: "A".to_string(),
                ticker: "A".to_string(),
                description: "A bet with 100% upside and 50% downside, with probabilities 50-50".to_string(),
                market_cap: 1e7,
                scenarios: vec![
                    Scenario {
                        thesis: "A1".to_string(),
                        intrinsic_value: 2e7,
                        probability: 0.5,
                    },
                    Scenario {
                        thesis: "A2".to_string(),
                        intrinsic_value: 5e6,
                        probability: 0.5,
                    },
                ],
            },
            Company {
                name: "B".to_string(),
                ticker: "B".to_string(),
                description: "A bet with 50% upside with 70% probability, and 30% downside with 30% probability".to_string(),
                market_cap: 1e7,
                scenarios: vec![
                    Scenario {
                        thesis: "B1".to_string(),
                        intrinsic_value: 1.5e7,
                        probability: 0.7,
                    },
                    Scenario {
                        thesis: "B2".to_string(),
                        intrinsic_value: 7e6,
                        probability: 0.3,
                    },
                ],
            },
        ];

        let candidates_and_fractions: Vec<(&Company, f64)> =
            vec![(&candidates[0], 0.5), (&candidates[1], 0.5)];

        let outcomes: Vec<Outcome> = vec![
            // Events A1 and B1
            Outcome {
                weighted_return: 0.75,
                probability: 0.35,
                company_returns: HashMap::from([("A".to_string(), 1.0), ("B".to_string(), 0.5)]),
            },
            // Events A1 and B2
            Outcome {
                weighted_return: 0.35,
                probability: 0.15,
                company_returns: HashMap::from([("A".to_string(), 1.0), ("B".to_string(), -0.3)]),
            },
            // Events A2 and B1
            Outcome {
                weighted_return: 0.0,
                probability: 0.35,
                company_returns: HashMap::from([("A".to_string(), -0.5), ("B".to_string(), 0.5)]),
            },
            // Events A2 and B1
            Outcome {
                weighted_return: -0.4,
                probability: 0.15,
                company_returns: HashMap::from([("A".to_string(), -0.5), ("B".to_string(), -0.3)]),
            },
        ];

        let jacobian = jacobian(&outcomes, candidates_and_fractions);

        assert!(
            (jacobian[(0, 0)] + 0.388256908).abs() < ASSERTION_TOLERANCE,
            "Jacobian at (0,0) is: {}",
            jacobian[(0, 0)]
        );
        assert!(
            (jacobian[(0, 1)] + 0.007451499).abs() < ASSERTION_TOLERANCE,
            "Jacobian at (0,1) is: {}",
            jacobian[(0, 1)]
        );
        assert!(
            (jacobian[(1, 0)] + 0.007451499).abs() < ASSERTION_TOLERANCE,
            "Jacobian at (1,0) is: {}",
            jacobian[(1, 0)]
        );
        assert!(
            (jacobian[(1, 1)] + 0.160978836).abs() < ASSERTION_TOLERANCE,
            "Jacobian at (1,1) is: {}",
            jacobian[(1, 1)]
        );
    }
}
