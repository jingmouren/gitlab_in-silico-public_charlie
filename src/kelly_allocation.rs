use std::any::Any;

use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use bitvec::view::BitView;
use nalgebra::{DMatrix, DVector};
use num_traits::pow;
use num_traits::pow::Pow;
use ordered_float::OrderedFloat;
use slog::{info, Logger};

use crate::analysis::{all_outcomes, worst_case_outcome, Outcome};
use crate::constraints::capital_loss_constraint::CapitalLossConstraint;
use crate::constraints::constraint::InequalityConstraint;
use crate::constraints::no_shorting_constraint::NoShortingConstraint;
use crate::model::capital_loss::CapitalLoss;
use crate::model::company::{Company, TOLERANCE};
use crate::model::errors::Error;
use crate::model::portfolio::{Portfolio, PortfolioCompany};

/// Tolerance for converging the solution during Newton-Raphson iteration. This is an absolute
/// tolerance, which may need to be modified into relative tolerance due to addition of constraints.
/// TODO: Think more
pub const SOLVER_TOLERANCE: f64 = 1e-6;

/// Relaxation factor used when updating solution vector in an iteration of the nonlinear loop.
const RELAXATION_FACTOR: f64 = 0.7;

/// Maximum number of iterations for the nonlinear solver.
pub const MAX_ITER: u32 = 1000;

/// Kelly allocator with an optional constraint for maximum loss of capital constraint. The
/// constraint may be inactive or active, which is figured out during the solution process.
pub struct KellyAllocator<'a> {
    logger: &'a Logger,
    max_iter: u32,
    inequality_constraints: Vec<Box<dyn InequalityConstraint>>,
}

impl<'a> KellyAllocator<'a> {
    /// Create a new instance of the [KellyAllocator] given the candidate companies, without
    /// constraints.
    pub fn new(logger: &'a Logger, max_iter: u32) -> KellyAllocator<'a> {
        KellyAllocator {
            logger,
            max_iter,
            inequality_constraints: vec![],
        }
    }

    /// Return a new [KellyAllocator] with a constraint for no shorting, for all company candidates.
    /// The contents of the original object are moved into the new one. Panics in case a single
    /// constraint of this type is already present.
    pub fn with_no_shorting_constraint(self, n_candidates: usize) -> KellyAllocator<'a> {
        info!(
            self.logger,
            "Setting no shorting constraints for all of the {n_candidates}."
        );

        if n_candidates < 1 {
            panic!("Can't make sense of no shorting constraints because there are {n_candidates} candidates.")
        }

        // Fractions are always the first set of unknowns in the system.
        let no_shorting_constraints: Vec<Box<dyn InequalityConstraint>> = (0..n_candidates)
            .map(|i| {
                Box::new(NoShortingConstraint { fraction_index: i })
                    as Box<dyn InequalityConstraint>
            })
            .collect();

        if self
            .inequality_constraints
            .iter()
            .any(|c| c.type_id() == no_shorting_constraints[0].type_id())
        {
            panic!(
                "Kelly allocator already initialized with no shorting constraints. Did you call \
                with_no_shorting_constraint twice?"
            )
        }

        let mut new_constraints = self.inequality_constraints;
        new_constraints.extend(no_shorting_constraints);

        KellyAllocator {
            logger: self.logger,
            max_iter: self.max_iter,
            inequality_constraints: new_constraints,
        }
    }

    /// Return a new [KellyAllocator] with a constraint for maximum permanent loss of capital.
    /// The contents of the original object are moved into the new one. Panics in case a constraint
    /// is already present. Panics in case we already have a constraint of this type.
    pub fn with_maximum_permanent_loss_constraint(
        self,
        max_permanent_loss_constraint: CapitalLoss,
    ) -> KellyAllocator<'a> {
        let constraint: Box<dyn InequalityConstraint> = Box::new(CapitalLossConstraint {
            fraction_of_capital: max_permanent_loss_constraint.fraction_of_capital,
            probability_of_loss: max_permanent_loss_constraint.probability_of_loss,
        });
        info!(
            self.logger,
            "Setting maximum permanent loss constraint: {:?}", max_permanent_loss_constraint
        );

        if self
            .inequality_constraints
            .iter()
            .any(|c| c.type_id() == constraint.type_id())
        {
            panic!(
                "Kelly allocator already initialized with a constraint representing maximum \
                permanent loss of capital. Did you call with_maximum_permanent_loss_constraint \
                twice?"
            )
        }

        let mut new_constraints = self.inequality_constraints;
        new_constraints.push(constraint);

        KellyAllocator {
            logger: self.logger,
            max_iter: self.max_iter,
            inequality_constraints: new_constraints,
        }
    }

    /// Calculates allocation factors (fractions) for each company based on the Kelly criterion, by
    /// solving M sets of N nonlinear equations using the Newton-Raphson algorithm where:
    /// - M is the number of systems to solve, equal to 2^N_IC, where N_IC is the number of
    ///   inequality constraints, because each inequality constraint may be active and inactive. If
    ///   there are no inequality constraints, only one system is solved.
    /// - N is the number of candidate companies plus the number of constraints.
    pub fn allocate(&self, candidates: Vec<Company>) -> Result<Portfolio, Error> {
        // Number of systems to solve is equal to 2^N_inequality_constraints
        let n_inequality_constraints: usize = self.inequality_constraints.len();
        let n_systems: usize = pow(2, n_inequality_constraints);
        info!(
            self.logger,
            "Need to solve 2^{n_inequality_constraints} = {n_systems} systems."
        );

        if n_systems > 255 {
            return Err(Error {
                code: "refusing-to-solve-more-than-255-systems".to_string(),
                message: format!(
                    "Solving more than 255 systems due to inequality constraints is \
                    prohibited because it hasn't been tested thoroughly, although it should work. \
                    You have {n_inequality_constraints} constraints resulting in {n_systems} \
                    systems to solve."
                ),
            });
        }

        // Size of each system is equal to number of companies + number of constraints
        let n_companies: usize = candidates.len();
        info!(
            self.logger,
            "Solving the Kelly allocation problem for {n_companies} companies."
        );

        let system_size = candidates.len() + n_inequality_constraints;
        info!(self.logger, "Size of each system is {system_size}.");

        // Initial guess for fractions assumes uniform allocation across all companies
        let uniform_fraction: f64 = 1.0 / n_companies as f64;

        // Get all outcomes for a list of candidates. Note that the fractions are not relevant here
        // since we only care about non-weighted company returns and probability.
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

        // Vector for collecting all viable solutions (unknown result vectors)
        let mut solutions: Vec<DVector<f64>> = Vec::with_capacity(n_systems);

        // Loop through all combinations, where the unsigned integer index is used to figure out
        // which constraint is active or inactive, based on its bit representation. Note that if
        // there are no constraints, we still have n_systems = 1. Example with four bits for
        // simplicity:
        // 0 = 0000 Everything is false (inactive)
        // 1 = 0001 First constraint is active, others are inactive
        // 2 = 0010 Second constraint is active, others are inactive
        // ...
        let mut all_error_strings: String = "".to_string();
        (0..n_systems).for_each(|index| {
            // Look at the bits of the integer to figure out whether a constraint is active.
            // Starting from least significant bit, indicating the status of first constraint.
            // Note that we only take first n_inequality_constraints bits which are the only ones
            // that are actually relevant (because a single usize is represented by 32 or 64 bits)
            let is_constraint_active: &BitSlice = index
                .view_bits::<Lsb0>()
                .split_at(n_inequality_constraints)
                .0;
            info!(
                self.logger,
                "Solving the {index}. system with following status of constraints:"
            );
            (0..n_inequality_constraints).for_each(|c_id| {
                if is_constraint_active[c_id] {
                    info!(self.logger, "    Constraint {c_id} is active.")
                } else {
                    info!(self.logger, "    Constraint {c_id} is inactive.")
                }
            });

            let result = self.solve_system(portfolio.clone(), &outcomes, is_constraint_active);

            // Check the result and:
            // 1. If the solution is not viable, ignore it. The solution is considered not viable
            //    when any of the slack variables associated with the _inactive_ inequality
            //    constraint is negative
            // 2. If the solution is viable, add it to the list
            // 3. If there was an error, simply ignore this solution. It might happen that we have
            //    other good solutions to pick from. TODO: Think more about when this can happen.
            match result {
                Ok(x) => {
                    // TODO: Extract a function for updating portfolio with a result
                    portfolio.companies.iter_mut().enumerate().for_each(|(i, pc)| pc.fraction = x[i]);

                    // Check whether all constraints are satisfied
                    self.inequality_constraints
                        .iter()
                        .enumerate()
                        .for_each(|(c_id, constraint)| {
                            info!(self.logger, "Constraint function value is: {:?}", constraint.function_value(&portfolio, 0.0));
                            if !constraint.is_satisfied(&portfolio) {
                                panic!(
                                    "Constraint {} is not satisfied. Constraint function value is: {}",
                                    c_id,
                                    constraint.function_value(&portfolio, 0.0)
                                )
                            }
                        });

                    if (0..n_inequality_constraints).any(|c_id| {
                        !is_constraint_active[c_id] && x[n_companies + c_id] < TOLERANCE
                    }) {
                        info!(
                            self.logger,
                            "Solution is not viable, skipping it. Solution vector: {x}."
                        );
                    } else {
                        info!(
                            self.logger,
                            "This is a viable solution. Adding it to the list of all solutions."
                        );
                        solutions.push(x)
                    }
                }
                Err(e) => {
                    all_error_strings.push_str(&format!("    {:?}: {:?}\n", index, e));
                    info!(
                        self.logger,
                        "Could not find a solution, skipping it. Error was {:?}", e
                    )
                }
            }
        });

        // Assume that the best solution is the one with the smallest possible risk of permanent
        // loss of capital, which is defined by the maximum probability weighted return of a
        // worst-case outcome: We're looking for maximum because worst-case outcome should be
        // negative.
        info!(self.logger, "{:?}", solutions);
        let best_solution = solutions.iter().max_by_key(|x| {
            // Update the portfolio with this solution vector
            let mut p = portfolio.clone();
            p.companies
                .iter_mut()
                .enumerate()
                .for_each(|(i, pc)| pc.fraction = x[i]);

            OrderedFloat(worst_case_outcome(&p, self.logger).probability_weighted_return)
        });

        match best_solution {
            Some(x) => {
                portfolio
                    .companies
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, pc)| pc.fraction = x[i]);
            }
            None => {
                return Err(Error {
                    code: "did-not-find-a-single-viable-solution".to_string(),
                    message: format!(
                        "Did not manage to find a single viable numerical solution. \
                         This may happen for multiple reasons. Check whether the input data would \
                         suggest a very strong bias towards a single/few investments. Check whether \
                         the constraints are too strict.\n\
                         Errors in individual solutions are {}:", all_error_strings
                    ),
                });
            }
        }

        Ok(portfolio)
    }

    /// Solves a system given a portfolio, all outcomes and constraint activity mask. The solution
    /// is found iteratively using the Newton-Raphson method since the resulting system is
    /// nonlinear. Constraints are added to the system based on their status (active/inactive).
    fn solve_system(
        &self,
        mut portfolio: Portfolio,
        outcomes: &[Outcome],
        is_constraint_active: &BitSlice,
    ) -> Result<DVector<f64>, Error> {
        let n_companies = portfolio.companies.len();
        let n_constraints = self.inequality_constraints.len();
        let n = n_companies + n_constraints;

        // Initialize vector of unknowns (x) with uniform fractions for companies, leaving potential
        // lagrange multipliers and slack variables initialized to zero (if n_constraints > 0)
        let mut x: DVector<f64> = DVector::from_element(n, 0.0);
        let uniform_fraction = 1.0 / n_companies as f64;
        (0..n_companies).for_each(|id| x[id] = uniform_fraction);

        let mut counter: u32 = 0;
        loop {
            // Update the fractions in the portfolio for calculating Kelly function and Jacobian
            portfolio
                .companies
                .iter_mut()
                .enumerate()
                .for_each(|(i, pc)| pc.fraction = x[i]);

            let mut jacobian: DMatrix<f64> = Self::criterion_jacobian(outcomes, &portfolio);
            let mut right_hand_side: DVector<f64> = -Self::criterion(outcomes, &portfolio);

            // Extend the matrix and RHS vector if we have constraints
            jacobian = jacobian.insert_columns(n_companies, n_constraints, 0.0);
            jacobian = jacobian.insert_rows(n_companies, n_constraints, 0.0);
            right_hand_side = right_hand_side.insert_rows(n_companies, n_constraints, 0.0);

            for cid in 0..n_constraints {
                let constraint: &dyn InequalityConstraint =
                    self.inequality_constraints[cid].as_ref();

                let d_constraint_d_fractions: DVector<f64> =
                    constraint.d_constraint_d_fractions(&portfolio);

                let offset_cid = n_companies + cid;

                // Constraint contribution is always added to the lower triangular row for this
                // constraint, regardless whether it's active or inactive
                for (eid, &elem) in d_constraint_d_fractions.iter().enumerate() {
                    jacobian[(eid, offset_cid)] = elem;
                }

                if is_constraint_active[cid] {
                    // Lagrange multiplier value from the previous iteration
                    let lambda = x[offset_cid];

                    // For active constraint, we have:
                    // 1. The upper triangular contribution (column) for this constraint.
                    // 2. Diagonal element of constraint equation remains zero.
                    // 3. The right-hand-side contribution for fraction equations.
                    for (eid, &elem) in d_constraint_d_fractions.iter().enumerate() {
                        jacobian[(offset_cid, eid)] = elem;
                        right_hand_side[eid] -= lambda * elem;
                    }

                    // 4. The right-hand side contribution for the constraint equation.
                    right_hand_side[offset_cid] -= constraint.function_value(&portfolio, 0.0);
                } else {
                    // For inactive constraint, we have:
                    // 1. The upper triangular column for this constraint remains 0.
                    // 2. Diagonal element of constraint equation is always 1.
                    // 3. The right-hand-side contribution for the constraint equations.
                    jacobian[(offset_cid, offset_cid)] = 1.0;

                    let slack_variable = x[offset_cid];
                    right_hand_side[offset_cid] -=
                        constraint.function_value(&portfolio, slack_variable);
                }
            }

            // Solve for delta_x and update the current solution vector
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

            let delta_x: DVector<f64> = inverse_jacobian * &right_hand_side;
            x += RELAXATION_FACTOR * &delta_x;

            // Convergence check (with Chebyshev/L-infinity norm)
            let residual = delta_x.abs().max();
            info!(
                self.logger,
                "Residual: {residual}. Performing convergence check."
            );
            if residual < SOLVER_TOLERANCE {
                info!(
                    self.logger,
                    "Newton-Raphson loop converged within {counter} iterations."
                );
                break;
            }

            // Maximum iterations check in case we diverge
            if counter >= self.max_iter {
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
            info!(self.logger, "Finished {counter} iteration.");
        }

        Ok(x)
    }

    /// Calculates the Kelly criterion given all outcomes and portfolio
    fn criterion(outcomes: &[Outcome], portfolio: &Portfolio) -> DVector<f64> {
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
    fn criterion_jacobian(outcomes: &[Outcome], portfolio: &Portfolio) -> DMatrix<f64> {
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
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::env::create_test_logger;
    use crate::model::company::Company;
    use crate::model::scenario::Scenario;
    use crate::utils::assert_close;

    use super::*;

    /// Make assertion tolerance the same as the fraction tolerance (no point in more accuracy)
    const ASSERTION_TOLERANCE: f64 = SOLVER_TOLERANCE;

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

        let kelly = KellyAllocator::criterion(&outcomes, &portfolio);

        assert_close!(0.011111111, kelly[0], ASSERTION_TOLERANCE);
        assert_close!(0.166666666, kelly[1], ASSERTION_TOLERANCE);
    }

    #[test]
    fn test_kelly_jacobian() {
        let test_candidates: Vec<Company> = generate_test_candidates();
        let (portfolio, outcomes): (Portfolio, Vec<Outcome>) = generate_test_data(&test_candidates);

        let jacobian = KellyAllocator::criterion_jacobian(&outcomes, &portfolio);

        assert_close!(-0.388256908, jacobian[(0, 0)], ASSERTION_TOLERANCE);
        assert_close!(-0.007451499, jacobian[(0, 1)], ASSERTION_TOLERANCE);
        assert_close!(-0.007451499, jacobian[(1, 0)], ASSERTION_TOLERANCE);
        assert_close!(-0.160978836, jacobian[(1, 1)], ASSERTION_TOLERANCE);
    }

    /// Asserts results for a simple allocation problem with two companies, each with two scenarios.
    #[test]
    fn test_allocate() {
        let logger = create_test_logger();
        let test_candidates: Vec<Company> = generate_test_candidates();
        let portfolio: Portfolio = KellyAllocator::new(&logger, MAX_ITER)
            .allocate(test_candidates)
            .unwrap();

        assert_eq!(portfolio.companies.len(), 2);
        assert_close!(
            0.3592665,
            portfolio.companies[0].fraction,
            ASSERTION_TOLERANCE
        );
        assert_close!(
            1.6299324,
            portfolio.companies[1].fraction,
            ASSERTION_TOLERANCE
        );
    }

    /// Same inputs as the above, but with a capital loss constraint that ends up being inactive.
    /// Therefore, the results is the same.
    #[test]
    fn test_allocate_with_capital_loss_constraint_inactive() {
        let logger = create_test_logger();
        let test_candidates: Vec<Company> = generate_test_candidates();
        let capital_loss_constraint = CapitalLoss {
            probability_of_loss: 0.15,
            fraction_of_capital: 0.50,
        };
        let allocator = KellyAllocator::new(&logger, MAX_ITER)
            .with_maximum_permanent_loss_constraint(capital_loss_constraint);
        let portfolio: Portfolio = allocator.allocate(test_candidates).unwrap();

        assert_eq!(portfolio.companies.len(), 2);
        assert_close!(
            0.3592665,
            portfolio.companies[0].fraction,
            ASSERTION_TOLERANCE
        );
        assert_close!(
            1.6299324,
            portfolio.companies[1].fraction,
            ASSERTION_TOLERANCE
        );
    }

    /// Tests that allocation with a fairly stringent capital loss constraint produces the same
    /// result as above, because the solution when the constraint is active produces a negative
    /// fraction (shorting).
    #[test]
    fn test_allocate_with_capital_loss_constraint_active() {
        let logger = create_test_logger();
        let test_candidates: Vec<Company> = generate_test_candidates();
        let capital_loss_constraint = CapitalLoss {
            probability_of_loss: 1e-5,
            fraction_of_capital: 0.1,
        };
        let portfolio: Portfolio = KellyAllocator::new(&logger, MAX_ITER)
            .with_maximum_permanent_loss_constraint(capital_loss_constraint)
            .allocate(test_candidates)
            .unwrap();

        assert_eq!(portfolio.companies.len(), 2);
        assert_close!(
            -0.254566,
            portfolio.companies[0].fraction,
            ASSERTION_TOLERANCE
        );
        assert_close!(
            0.707116,
            portfolio.companies[1].fraction,
            ASSERTION_TOLERANCE
        );
    }

    #[test]
    fn test_allocate_with_one_short_result() {
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
        let portfolio: Portfolio = KellyAllocator::new(&logger, MAX_ITER)
            .allocate(test_candidates)
            .unwrap();

        assert_eq!(portfolio.companies.len(), 3);
        assert_close!(
            0.323636,
            portfolio.companies[0].fraction,
            ASSERTION_TOLERANCE
        );
        assert_close!(
            1.535812,
            portfolio.companies[1].fraction,
            ASSERTION_TOLERANCE
        );
        assert_close!(
            -0.323635,
            portfolio.companies[2].fraction,
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
        let portfolio: Portfolio = KellyAllocator::new(&logger, MAX_ITER)
            .allocate(test_candidates)
            .unwrap();

        assert_eq!(portfolio.companies.len(), 1);
        assert_close!(0.5, portfolio.companies[0].fraction, ASSERTION_TOLERANCE);
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
        let portfolio: Portfolio = KellyAllocator::new(&logger, MAX_ITER)
            .allocate(test_candidates)
            .unwrap();

        assert_eq!(portfolio.companies.len(), 1);
        assert_close!(
            89.988889,
            portfolio.companies[0].fraction,
            ASSERTION_TOLERANCE
        );
    }

    #[test]
    fn test_allocate_returns_an_error_when_maximum_iterations_exceeded() {
        let logger = create_test_logger();
        let e = KellyAllocator::new(&logger, 1)
            .allocate(generate_test_candidates())
            .err()
            .unwrap();
        assert_eq!(e.code, "did-not-find-a-single-viable-solution");
        assert!(e
            .message
            .contains("Did not manage to find a single viable numerical solution."));
        assert!(e
            .message
            .contains("Did not manage to find the numerical solution."));
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
        let e = KellyAllocator::new(&logger, MAX_ITER)
            .allocate(test_candidates)
            .err()
            .unwrap();
        assert_eq!(e.code, "did-not-find-a-single-viable-solution");
        assert!(e
            .message
            .contains("Did not manage to find a single viable numerical solution."));
        assert!(e
            .message
            .contains("Did not manage to find the numerical solution."));
    }
}
