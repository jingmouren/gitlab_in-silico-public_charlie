use crate::analysis::analysis::Outcome;
use crate::model::company::Company;
use nalgebra::DMatrix;

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

    #[test]
    fn test_jacobian() {
        // TODO: Make a realistic example

        let outcomes: Vec<Outcome> = vec![
            Outcome {
                portfolio_return: 0.43,
                probability: 0.6,
            },
            Outcome {
                portfolio_return: 0.23,
                probability: 0.4,
            },
        ];

        let candidates: Candidates = vec![
            Company {
                name: "Fair coin flip".to_string(),
                ticker: "A".to_string(),
                description: "Something we should never invest into".to_string(),
                market_cap: 1e6,
                scenarios: vec![
                    Scenario {
                        thesis: "Head".to_string(),
                        intrinsic_value: 2e6,
                        probability: 0.5,
                    },
                    Scenario {
                        thesis: "Tail".to_string(),
                        intrinsic_value: 0.0,
                        probability: 0.5,
                    },
                ],
            },
            Company {
                name: "Biased coin flip".to_string(),
                ticker: "B".to_string(),
                description: "A not-so-fair coin flip".to_string(),
                market_cap: 1e6,
                scenarios: vec![
                    Scenario {
                        thesis: "Head".to_string(),
                        intrinsic_value: 2e6,
                        probability: 0.6,
                    },
                    Scenario {
                        thesis: "Tail".to_string(),
                        intrinsic_value: 0.0,
                        probability: 0.4,
                    },
                ],
            },
        ];

        let jacobian = jacobian(&outcomes, &candidates);

        for i in 0..candidates.len() {
            for j in 0..candidates.len() {
                assert_eq!(jacobian[(i, j)], 0.0)
            }
        }
    }
}
