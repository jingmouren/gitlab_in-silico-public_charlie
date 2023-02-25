use crate::analysis::{all_outcomes, Outcome};
use crate::model::company::Company;
use crate::Portfolio;
use itertools::Itertools;
use nalgebra::{DMatrix, DVector};
use num_traits::pow::Pow;
use std::collections::HashMap;

/// Tolerance for converging the fraction during Newton-Raphson iteration. Corresponds to 1%, which
/// is more than enough given that the real uncertainty lies in the input data and not here.
const FRACTION_TOLERANCE: f64 = 1e-2;
const MAX_ITER: u32 = 1000;

/// Calculates allocation factors (fractions) for each company based on the Kelly criterion, by
/// solving N nonlinear equations (N = number of candidates) using the Newton-Raphson algorithm
pub fn kelly_criterion_allocate(candidates: Vec<Company>) -> Portfolio {
    // Initial guess for fractions assumes uniform allocation across all companies
    let n_companies: usize = candidates.len();
    let uniform_fraction: f64 = 1.0 / n_companies as f64;
    let mut fractions: DVector<f64> = DVector::from_element(candidates.len(), uniform_fraction);

    // Create a portfolio out of candidates, all with the same fractions (which is irrelevant here)
    let mut portfolio: Portfolio =
        HashMap::from_iter(candidates.iter().map(|c| ((*c).clone(), uniform_fraction)));

    // Get all outcomes
    let outcomes: Vec<Outcome> = all_outcomes(&portfolio);

    let mut counter: u32 = 0;
    loop {
        // Calculate the Jacobian with the latest fractions for all companies
        let candidates_and_fractions: Vec<(&Company, f64)> = candidates
            .iter()
            .enumerate()
            .map(|(i, c)| (c, fractions[i]))
            .collect_vec();

        let jacobian: DMatrix<f64> = kelly_jacobian(&outcomes, &candidates_and_fractions);
        let right_hand_side: DVector<f64> = -kelly(&outcomes, &candidates_and_fractions);

        // TODO: Error handling
        let delta_f: DVector<f64> = jacobian.try_inverse().unwrap() * &right_hand_side;
        fractions += &delta_f;

        // Convergence check (with Chebyshev/L-infinity norm)
        if (delta_f).abs().max() < FRACTION_TOLERANCE {
            println!("Newton-Raphson loop converged within {counter} iterations");
            break;
        }

        // Maximum iterations check in case we diverge
        if counter >= MAX_ITER {
            panic!("Convergence not achieved within maximum number of iterations.")
        }

        counter += 1
    }

    // Check whether we got a negative fraction, which implies shorting. This should not happen if
    // we filter out candidates with negative expected value (at least I think, I'm not 100% sure
    // since I didn't work on a mathematical proof: it's just my feeling)
    if fractions.min() < 0.0 {
        panic!(
            "Found at least one negative fraction, which implies shorting. This is not supported. \
            Fractions are: {fractions}."
        )
    }

    // Normalize the fractions such that their sum is equal to one. This essentially means that we
    // do not want to use leverage.
    // TODO: Pretty sure that implicitly constraining with e.g. Lagrange multipliers to have
    //  sum(f) = 1 is equivalent to just normalizing after solving, but not 100% sure. Think more.
    let sum_fractions = fractions.iter().sum();
    if sum_fractions > 1.0 {
        println!(
            "Sum of the fractions after the solution is {sum_fractions}, which is greater than \
            one. This implies use of leverage. Normalizing the fractions to avoid leverage."
        );
        fractions /= sum_fractions;
    }

    // Update the fractions and return the portfolio
    candidates.into_iter().enumerate().for_each(|(i, c)| {
        portfolio.insert(c, fractions[i]);
    });

    portfolio
}

/// Calculates the Kelly criterion given all outcomes, companies and their fractions
fn kelly(outcomes: &[Outcome], candidates_and_fractions: &Vec<(&Company, f64)>) -> DVector<f64> {
    let n_companies = candidates_and_fractions.len();

    let kelly: DVector<f64> = DVector::from_iterator(
        n_companies,
        candidates_and_fractions.iter().map(|(company, _)| {
            outcomes
                .iter()
                .map(|o| {
                    o.probability * o.company_returns[&company.ticker]
                        / (1.0
                            + candidates_and_fractions
                                .iter()
                                .map(|(c, f)| f * o.company_returns[&c.ticker])
                                .sum::<f64>())
                })
                .sum::<f64>()
        }),
    );

    kelly
}

/// Calculates the Jacobian for the Kelly function given all outcomes, companies and their fractions
fn kelly_jacobian(
    outcomes: &[Outcome],
    candidates_and_fractions: &Vec<(&Company, f64)>,
) -> DMatrix<f64> {
    let n_companies: usize = candidates_and_fractions.len();
    let mut jacobian: DMatrix<f64> = DMatrix::zeros(n_companies, n_companies);

    // Note: Jacobian for this system is symmetric, that's why we loop only over the upper triangle
    for row_index in 0..n_companies {
        for column_index in row_index..n_companies {
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
                .sum::<f64>();

            // Set lower triangle. Also overrides the diagonal with the same value unnecessarily,
            // but seems more elegant compared to an if statement
            jacobian[(column_index, row_index)] = jacobian[(row_index, column_index)];
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

    const ASSERTION_TOLERANCE: f64 = 1e-6;

    /// Helper function for defining test candidates
    fn generate_test_candidates() -> Vec<Company> {
        vec![
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
        ]
    }

    /// Helper function for generating test data used in unit tests
    fn generate_test_data(test_candidates: &Vec<Company>) -> (Vec<(&Company, f64)>, Vec<Outcome>) {
        let candidates_and_fractions: Vec<(&Company, f64)> =
            vec![(&test_candidates[0], 0.5), (&test_candidates[1], 0.5)];

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

        (candidates_and_fractions, outcomes)
    }

    #[test]
    fn test_kelly() {
        let test_candidates: Vec<Company> = generate_test_candidates();
        let (candidates_and_fractions, outcomes): (Vec<(&Company, f64)>, Vec<Outcome>) =
            generate_test_data(&test_candidates);

        let kelly = kelly(&outcomes, &candidates_and_fractions);

        assert!(
            (kelly[0] - 0.011111111).abs() < ASSERTION_TOLERANCE,
            "Kelly value at 0 is: {}",
            kelly[0]
        );

        assert!(
            (kelly[1] - 0.166666666).abs() < ASSERTION_TOLERANCE,
            "Kelly value at 1 is: {}",
            kelly[1]
        );
    }

    #[test]
    fn test_kelly_jacobian() {
        let test_candidates: Vec<Company> = generate_test_candidates();
        let (candidates_and_fractions, outcomes): (Vec<(&Company, f64)>, Vec<Outcome>) =
            generate_test_data(&test_candidates);

        let jacobian = kelly_jacobian(&outcomes, &candidates_and_fractions);

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

    #[test]
    fn test_allocate() {
        let test_candidates: Vec<Company> = generate_test_candidates();
        let portfolio: Portfolio = kelly_criterion_allocate(test_candidates);

        let test_candidates_not_moved: Vec<Company> = generate_test_candidates();

        assert!((portfolio[&test_candidates_not_moved[0]] - 0.180609).abs() < ASSERTION_TOLERANCE);
        assert!((portfolio[&test_candidates_not_moved[1]] - 0.819391).abs() < ASSERTION_TOLERANCE);
    }

    #[test]
    #[should_panic(
        expected = "Found at least one negative fraction, which implies shorting. This is not \
            supported. Fractions are: "
    )]
    fn test_allocate_panics_with_a_candidate_for_shorting() {
        let mut test_candidates: Vec<Company> = generate_test_candidates();
        test_candidates.push(Company {
            name: "Stupid investment".to_string(),
            ticker: "SI".to_string(),
            description: "A bet with 50% upside and 100% downside, with probabilities 50-50"
                .to_string(),
            market_cap: 1e7,
            scenarios: vec![
                Scenario {
                    thesis: "Ok".to_string(),
                    intrinsic_value: 1.5e7,
                    probability: 0.5,
                },
                Scenario {
                    thesis: "Bad".to_string(),
                    intrinsic_value: 0.0,
                    probability: 0.5,
                },
            ],
        });
        kelly_criterion_allocate(test_candidates);
    }
}
