use crate::analysis::{all_outcomes, Outcome};
use crate::model::company::Company;
use crate::model::errors::Error;
use crate::model::portfolio::{Portfolio, PortfolioCompany};
use nalgebra::{DMatrix, DVector};
use num_traits::pow::Pow;
use slog::{info, Logger};

/// Tolerance for converging the fraction during Newton-Raphson iteration. Corresponds to 1%, which
/// is more than enough given that the real uncertainty lies in the input data and not here.
pub const FRACTION_TOLERANCE: f64 = 1e-2;

/// Relaxation factor used when updating solution vector in an iteration of the nonlinear loop
const RELAXATION_FACTOR: f64 = 0.7;

/// Maximum number of iterations for the nonlinear solver
pub const MAX_ITER: u32 = 1000;

/// Calculates allocation factors (fractions) for each company based on the Kelly criterion, by
/// solving N nonlinear equations (N = number of candidates) using the Newton-Raphson algorithm
pub fn kelly_allocate(
    candidates: Vec<Company>,
    max_iter: u32,
    logger: &Logger,
) -> Result<Portfolio, Error> {
    let n_companies: usize = candidates.len();
    info!(
        logger,
        "Solving the Kelly allocation equations for {n_companies} companies"
    );

    // Initial guess for fractions assumes uniform allocation across all companies
    let uniform_fraction: f64 = 1.0 / n_companies as f64;
    let mut fractions: DVector<f64> = DVector::from_element(n_companies, uniform_fraction);

    // Get all outcomes for a list of candidates. Note that the fractions are not relevant here
    // since we only care about non-weighted company returns and probability
    let mut portfolio: Portfolio = Portfolio {
        companies: candidates
            .into_iter()
            .map(|c| PortfolioCompany {
                company: c,
                fraction: uniform_fraction,
            })
            .collect(),
    };
    let outcomes: Vec<Outcome> = match all_outcomes(&portfolio) {
        Ok(o) => o,
        Err(e) => return Err(e),
    };

    info!(logger, "Starting Newton-Raphson loop.");
    let mut counter: u32 = 0;
    loop {
        // Update the fractions in the portfolio for calculating Kelly function and Jacobian
        portfolio
            .companies
            .iter_mut()
            .enumerate()
            .for_each(|(i, pc)| pc.fraction = fractions[i]);

        // Calculate the Jacobian with the latest fractions for all companies
        info!(logger, "Calculating the Jacobian.");
        let jacobian: DMatrix<f64> = kelly_criterion_jacobian(&outcomes, &portfolio);
        let right_hand_side: DVector<f64> = -kelly_criterion(&outcomes, &portfolio);

        // Solve for delta_f and update the fractions in the portfolio
        info!(logger, "Inverting the Jacobian.");
        let inverse_jacobian: DMatrix<f64> = match jacobian.try_inverse() {
            Some(s) => s,
            None => return Err(Error {
                code: "jacobian-inversion-failed".to_string(),
                message:
                    "Did not manage to find the numerical solution. This may happen if the input \
                    data would suggest a very strong bias towards a single/few investments. \
                    Check your input."
                        .to_string(),
            }),
        };

        info!(logger, "Calculating new fractions.");
        let delta_f: DVector<f64> = inverse_jacobian * &right_hand_side;
        fractions += RELAXATION_FACTOR * &delta_f;

        // Convergence check (with Chebyshev/L-infinity norm)
        info!(logger, "Performing convergence check...");
        if delta_f.abs().max() < FRACTION_TOLERANCE {
            info!(
                logger,
                "Newton-Raphson loop converged within {counter} iterations"
            );
            break;
        }

        // Maximum iterations check in case we diverge
        if counter >= max_iter {
            return Err(Error {
                code: "nonlinear-loop-didnt-converge".to_string(),
                message:
                    "Did not manage to find the numerical solution. This may happen if the input \
                    data would suggest a very strong bias towards a single/few investments. \
                    Check your input."
                        .to_string(),
            });
        }

        counter += 1;
        info!(logger, "Finished {counter} iteration");
    }

    // Check whether we got a negative fraction, which implies shorting. This should not happen if
    // we filter out candidates with negative expected value (at least I think, I'm not 100% sure
    // since I didn't work on a mathematical proof: it's just my feeling)
    info!(logger, "Checking for negative fractions.");
    if fractions.min() < 0.0 {
        info!(
            logger,
            "Encountered a negative fraction, returning an error."
        );
        return Err(Error {
            code: "negative-fraction-after-solution".to_string(),
            message: format!(
                "Found at least one negative fraction, which implies shorting. This \
            should not happen. Fractions are: {fractions}."
            ),
        });
    } else {
        info!(logger, "All fractions are non-negative.")
    }

    // Normalize the fractions such that their sum is equal to one. This essentially means that we
    // do not want to use leverage.
    // TODO: Not sure whether implicitly constraining with e.g. Lagrange multipliers to have
    //  sum(f) = 1 is equivalent to just normalizing after solving. Think more.
    let sum_fractions = fractions.sum();
    if sum_fractions > 1.0 {
        info!(
            logger,
            "Sum of the fractions after the solution is {sum_fractions}, which is greater than \
            one. This implies use of leverage. Normalizing the fractions to avoid leverage."
        );
        fractions /= sum_fractions;
    }

    // Update the fractions in portfolio and return
    portfolio
        .companies
        .iter_mut()
        .enumerate()
        .for_each(|(i, pc)| pc.fraction = fractions[i]);

    info!(
        logger,
        "Optimal allocation based on Kelly criterion calculated. Returning."
    );
    Ok(portfolio)
}

/// Calculates the Kelly criterion given all outcomes and portfolio
fn kelly_criterion(outcomes: &[Outcome], portfolio: &Portfolio) -> DVector<f64> {
    let n_companies = portfolio.companies.len();

    let kelly: DVector<f64> = DVector::from_iterator(
        n_companies,
        portfolio.companies.iter().map(|pc_outer| {
            outcomes
                .iter()
                .map(|o| {
                    o.probability * o.company_returns[&pc_outer.company.ticker]
                        / (1.0
                            + portfolio
                                .companies
                                .iter()
                                .map(|pc| pc.fraction * o.company_returns[&pc.company.ticker])
                                .sum::<f64>())
                })
                .sum::<f64>()
        }),
    );

    kelly
}

/// Calculates the Jacobian for the Kelly function given all outcomes and portfolio
fn kelly_criterion_jacobian(outcomes: &[Outcome], portfolio: &Portfolio) -> DMatrix<f64> {
    let n_companies: usize = portfolio.companies.len();
    let mut jacobian: DMatrix<f64> = DMatrix::zeros(n_companies, n_companies);

    // Note: Jacobian for this system is symmetric, that's why we loop only over the upper triangle
    for row_index in 0..n_companies {
        for column_index in row_index..n_companies {
            let row_company: &Company = &portfolio.companies[row_index].company;
            let column_company: &Company = &portfolio.companies[column_index].company;

            jacobian[(row_index, column_index)] = -outcomes
                .iter()
                .map(|o| {
                    o.probability
                        * o.company_returns[&row_company.ticker]
                        * o.company_returns[&column_company.ticker]
                        * (1.0
                            + portfolio
                                .companies
                                .iter()
                                .map(|pc| pc.fraction * o.company_returns[&pc.company.ticker])
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
    use crate::env::create_test_logger;
    use crate::model::company::Company;
    use crate::model::scenario::Scenario;
    use crate::utils::assert_close;
    use std::collections::HashMap;

    /// Make assertion tolerance the same as the fraction tolerance (no point in more accuracy)
    const ASSERTION_TOLERANCE: f64 = FRACTION_TOLERANCE;

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
    fn generate_test_data(test_candidates: &Vec<Company>) -> (Portfolio, Vec<Outcome>) {
        let portfolio: Portfolio = Portfolio {
            companies: vec![
                PortfolioCompany {
                    company: test_candidates[0].clone(),
                    fraction: 0.5,
                },
                PortfolioCompany {
                    company: test_candidates[1].clone(),
                    fraction: 0.5,
                },
            ],
        };

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

        (portfolio, outcomes)
    }

    #[test]
    fn test_kelly() {
        let test_candidates: Vec<Company> = generate_test_candidates();
        let (portfolio, outcomes): (Portfolio, Vec<Outcome>) = generate_test_data(&test_candidates);

        let kelly = kelly_criterion(&outcomes, &portfolio);

        assert_close!(0.011111111, kelly[0], ASSERTION_TOLERANCE);
        assert_close!(0.166666666, kelly[1], ASSERTION_TOLERANCE);
    }

    #[test]
    fn test_kelly_jacobian() {
        let test_candidates: Vec<Company> = generate_test_candidates();
        let (portfolio, outcomes): (Portfolio, Vec<Outcome>) = generate_test_data(&test_candidates);

        let jacobian = kelly_criterion_jacobian(&outcomes, &portfolio);

        assert_close!(-0.388256908, jacobian[(0, 0)], ASSERTION_TOLERANCE);
        assert_close!(-0.007451499, jacobian[(0, 1)], ASSERTION_TOLERANCE);
        assert_close!(-0.007451499, jacobian[(1, 0)], ASSERTION_TOLERANCE);
        assert_close!(-0.160978836, jacobian[(1, 1)], ASSERTION_TOLERANCE);
    }

    #[test]
    fn test_allocate() {
        let logger = create_test_logger();
        let test_candidates: Vec<Company> = generate_test_candidates();
        let portfolio: Portfolio = kelly_allocate(test_candidates, MAX_ITER, &logger).unwrap();

        assert_eq!(portfolio.companies.len(), 2);
        assert_close!(
            0.181507,
            portfolio.companies[0].fraction,
            ASSERTION_TOLERANCE
        );
        assert_close!(
            0.818493,
            portfolio.companies[1].fraction,
            ASSERTION_TOLERANCE
        );
    }

    #[test]
    fn test_allocate_for_a_single_company() {
        let test_candidates: Vec<Company> = vec![Company {
            name: "A".to_string(),
            ticker: "A".to_string(),
            description: "A bet with 100% upside and 50% downside, with probabilities 50-50"
                .to_string(),
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
        }];

        let logger = create_test_logger();
        let portfolio: Portfolio = kelly_allocate(test_candidates, MAX_ITER, &logger).unwrap();

        assert_eq!(portfolio.companies.len(), 1);
        assert_close!(
            0.502603,
            portfolio.companies[0].fraction,
            ASSERTION_TOLERANCE
        );
    }

    #[test]
    fn test_allocate_for_a_single_company_stiff_system() {
        let test_candidates: Vec<Company> = vec![Company {
            name: "A".to_string(),
            ticker: "A".to_string(),
            description: "A bet with 10x upside and 1% downside, with probabilities 90-10"
                .to_string(),
            market_cap: 1e7,
            scenarios: vec![
                Scenario {
                    thesis: "A1".to_string(),
                    intrinsic_value: 1e8,
                    probability: 0.9,
                },
                Scenario {
                    thesis: "A2".to_string(),
                    intrinsic_value: 0.99e7,
                    probability: 0.1,
                },
            ],
        }];

        let logger = create_test_logger();
        let portfolio: Portfolio = kelly_allocate(test_candidates, MAX_ITER, &logger).unwrap();

        assert_eq!(portfolio.companies.len(), 1);
        assert_close!(1.0, portfolio.companies[0].fraction, ASSERTION_TOLERANCE);
    }

    #[test]
    fn test_allocate_returns_an_error_when_maximum_iterations_exceeded() {
        let logger = create_test_logger();
        let e = kelly_allocate(generate_test_candidates(), 1, &logger)
            .err()
            .unwrap();
        assert_eq!(e.code, "nonlinear-loop-didnt-converge");
        assert!(e
            .message
            .contains("Did not manage to find the numerical solution."));
    }

    #[test]
    fn test_allocate_returns_an_error_with_a_candidate_for_shorting() {
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

        let logger = create_test_logger();
        let e = kelly_allocate(test_candidates, MAX_ITER, &logger)
            .err()
            .unwrap();
        assert_eq!(e.code, "negative-fraction-after-solution");
        assert!(e
            .message
            .contains("Found at least one negative fraction, which implies shorting."));
    }

    #[test]
    fn test_allocate_returns_an_error_with_a_candidate_with_no_downside() {
        let mut test_candidates: Vec<Company> = generate_test_candidates();
        test_candidates.push(Company {
            name: "Best investment that implies infinite bet".to_string(),
            ticker: "BI".to_string(),
            description: "A bet with 10x upside and no downside".to_string(),
            market_cap: 1.0e7,
            scenarios: vec![
                Scenario {
                    thesis: "10x upside".to_string(),
                    intrinsic_value: 1.0e8,
                    probability: 0.5,
                },
                Scenario {
                    thesis: "No downside".to_string(),
                    intrinsic_value: 1.0e7,
                    probability: 0.5,
                },
            ],
        });

        let logger = create_test_logger();
        let e = kelly_allocate(test_candidates, MAX_ITER, &logger)
            .err()
            .unwrap();
        assert_eq!(e.code, "jacobian-inversion-failed");
        assert!(e
            .message
            .contains("Did not manage to find the numerical solution."));
    }
}
