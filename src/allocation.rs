use crate::analysis::Outcome;
use crate::model::company::Company;
use nalgebra::DMatrix;

/// Calculates the Jacobian given all outcomes, companies and their fractions
pub fn _jacobian(
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
                })
                .sum::<f64>()
        }
    }

    jacobian
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::company::Company;
    use crate::model::scenario::Scenario;
    use std::collections::HashMap;

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

        //let outcomes: Vec<Outcome> = vec![
        //    // Event A1 and B1
        //    Outcome {
        //        portfolio_return: 0.43,
        //        probability: 0.35,
        //        company_returns: HashMap::from([
        //            ("Norway", 25),
        //            ("Denmark", 24),
        //            ("Iceland", 12),
        //        ])
        //    },
        //    Outcome {
        //        portfolio_return: 0.23,
        //        probability: 0.4,
        //    },
        //];
        //
        //
        //let jacobian = jacobian(&outcomes, &candidates);
        //
        //for i in 0..candidates.len() {
        //    for j in 0..candidates.len() {
        //        assert_eq!(jacobian[(i, j)], 0.0)
        //    }
        //}
    }
}
