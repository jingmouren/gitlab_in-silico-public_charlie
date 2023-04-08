use crate::model::company::Ticker;
use crate::model::errors::Error;
use crate::model::portfolio::Portfolio;
use ordered_float::OrderedFloat;
use slog::{info, Logger};
use std::collections::HashMap;

/// An outcome consists of its probability and portfolio return
#[derive(Debug)]
pub struct Outcome {
    pub weighted_return: f64,
    pub probability: f64,
    pub company_returns: HashMap<Ticker, f64>,
}

/// Returns all possible outcomes (expected portfolio return and associated probability)
pub fn all_outcomes(portfolio: &Portfolio) -> Result<Vec<Outcome>, Error> {
    // Number of different outcomes is a product of number of all scenarios for all companies
    let n_outcomes = if !portfolio.companies.is_empty() {
        portfolio
            .companies
            .iter()
            .map(|pc| pc.company.scenarios.len())
            .product()
    } else {
        0
    };

    if n_outcomes > 50000 {
        return Err(Error {
            code: "more-than-fifty-thousand-outcomes".to_string(),
            message: format!(
                "You have {n_outcomes} different outcomes for your portfolio. This \
            software is designed for a focused investment strategy, and it seems you have too many \
            companies or too many scenarios for companies.",
            ),
        });
    }

    // Mutable data that's populated/modified within the loop below
    // 1. Vectors for all outcomes
    let mut outcomes: Vec<Outcome> = Vec::with_capacity(n_outcomes);

    // 2. Helper vectors keeping track of current indices for scenarios of all companies
    let mut scenario_indices: Vec<usize> = vec![0; portfolio.companies.len()];
    let n_scenarios: Vec<usize> = portfolio
        .companies
        .iter()
        .map(|pc| pc.company.scenarios.len())
        .collect();

    // Start filling in outcomes until all are collected
    while outcomes.len() != n_outcomes {
        // 1. Calculate the outcome by summing up scenarios for all companies
        // Note: Probability is initialized with 1.0 since we multiply to get joint probability
        let mut outcome = Outcome {
            weighted_return: 0.0,
            probability: 1.0,
            company_returns: HashMap::with_capacity(portfolio.companies.len()),
        };

        portfolio
            .companies
            .iter()
            .enumerate()
            .for_each(|(ticker_id, pc)| {
                let scenario_id = scenario_indices[ticker_id];
                let c = &pc.company;
                let s = &c.scenarios[scenario_id];

                let company_return = (s.intrinsic_value - c.market_cap) / c.market_cap;
                outcome.weighted_return += pc.fraction * company_return;
                outcome.probability *= s.probability;
                outcome
                    .company_returns
                    .insert(c.ticker.clone(), company_return);
            });

        // 2. Append the calculated outcome to the list of outcomes
        outcomes.push(outcome);

        // 3. Increment a single index to prepare for the next iteration
        for (i, scenario_id) in scenario_indices.iter_mut().enumerate() {
            if *scenario_id + 1 == n_scenarios[i] {
                // We have exhausted the index for this company, set to zero and continue the
                // loop in order to start incrementing the next index
                *scenario_id = 0;
                continue;
            } else {
                // Increment the first non-overflowing index and break out
                *scenario_id += 1;
                break;
            }
        }
    }

    Ok(outcomes)
}

/// Calculates expected return of a portfolio
pub fn expected_return(portfolio: &Portfolio, logger: &Logger) -> f64 {
    let expected_return: f64 = portfolio
        .companies
        .iter()
        .map(|pc| {
            let market_cap = pc.company.market_cap;
            pc.company
                .scenarios
                .iter()
                .map(|s| {
                    pc.fraction * s.probability * (s.intrinsic_value - market_cap) / market_cap
                })
                .sum::<f64>()
        })
        .sum();

    info!(
        logger,
        "For every 1 dollar invested, we expect to end up with {:.2} dollars",
        1.0 + expected_return
    );

    expected_return
}

/// Finds an outcome with maximum loss of capital and reports its probability
pub fn worst_case_outcome<'a>(outcomes: &'a [Outcome], logger: &Logger) -> &'a Outcome {
    info!(logger, "Searching for worst case outcome.");
    let worst_case_outcome = outcomes
        .iter()
        .min_by_key(|o| OrderedFloat(o.weighted_return))
        .unwrap_or_else(|| {
            panic!(
                "Did not manage to find the worst case outcome in the list of outcomes: {:?}",
                outcomes
            )
        });

    info!(
        logger,
        "Worst case outcome implies permanent loss of {:.1}% of invested assets with probability {:.6}%",
        100.0 * worst_case_outcome.weighted_return,
        100.0 * worst_case_outcome.probability
    );

    worst_case_outcome
}

/// Calculates the cumulative probability of losing money
pub fn cumulative_probability_of_loss(outcomes: &[Outcome], logger: &Logger) -> f64 {
    let cumulative_probability_of_loss = outcomes
        .iter()
        .filter(|o| o.weighted_return < 0.0)
        .map(|o| o.probability)
        .sum();

    info!(
        logger,
        "Cumulative probability of loss of capital is {:.3}%",
        100.0 * cumulative_probability_of_loss
    );

    cumulative_probability_of_loss
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::env::create_test_logger;
    use crate::model::company;
    use crate::model::company::Company;
    use crate::model::portfolio::{Portfolio, PortfolioCompany};
    use crate::model::scenario::Scenario;
    use crate::utils::assert_close;

    impl PartialEq<Self> for Outcome {
        fn eq(&self, other: &Self) -> bool {
            ((self.weighted_return - other.weighted_return).abs() < company::TOLERANCE)
                && ((self.probability - other.probability).abs() < company::TOLERANCE)
                && (self.company_returns.iter().all(|(ticker, ret)| {
                    (ret - other.company_returns[ticker]).abs() < company::TOLERANCE
                }))
        }
    }

    /// A helper function that creates portfolio with three assets used in a couple of tests
    fn get_test_portfolio_with_three_assets() -> Portfolio {
        let test_portfolio: Portfolio = Portfolio {
            companies: vec![
                // Fair coin flip
                PortfolioCompany {
                    company: Company {
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
                    fraction: 0.2,
                },
                // Biased coin flip with 30% allocation
                PortfolioCompany {
                    company: Company {
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
                    fraction: 0.3,
                },
                // Something with only upside 50% allocation
                PortfolioCompany {
                    company: Company {
                        name: "Something with only upside".to_string(),
                        ticker: "C".to_string(),
                        description: "Shouldn't lose money here because of xyz".to_string(),
                        market_cap: 1e8,
                        scenarios: vec![
                            Scenario {
                                thesis: "Double".to_string(),
                                intrinsic_value: 2e8,
                                probability: 0.3,
                            },
                            Scenario {
                                thesis: "50 percent up".to_string(),
                                intrinsic_value: 1.5e8,
                                probability: 0.3,
                            },
                            Scenario {
                                thesis: "Same as now".to_string(),
                                intrinsic_value: 1e8,
                                probability: 0.4,
                            },
                        ],
                    },
                    fraction: 0.5,
                },
            ],
        };

        test_portfolio
    }

    #[test]
    fn test_expected_value_single_fair_coin_flip() {
        let test_portfolio: Portfolio = Portfolio {
            companies: vec![PortfolioCompany {
                company: Company {
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
                fraction: 1.0,
            }],
        };

        let logger = create_test_logger();
        assert_close!(
            0.0,
            expected_return(&test_portfolio, &logger),
            company::TOLERANCE
        );
    }

    #[test]
    fn test_expected_value_single_biased_coin_flip() {
        let test_portfolio: Portfolio = Portfolio {
            companies: vec![PortfolioCompany {
                company: Company {
                    name: "Biased coin flip".to_string(),
                    ticker: "B".to_string(),
                    description: "A not-so-fair coin flip".to_string(),
                    market_cap: 1e6,
                    scenarios: vec![
                        Scenario {
                            thesis: "Head".to_string(),
                            intrinsic_value: 2e6,
                            probability: 0.8,
                        },
                        Scenario {
                            thesis: "Tail".to_string(),
                            intrinsic_value: 0.0,
                            probability: 0.2,
                        },
                    ],
                },
                fraction: 1.0,
            }],
        };

        let logger = create_test_logger();
        assert_close!(
            0.6,
            expected_return(&test_portfolio, &logger),
            company::TOLERANCE
        );
    }

    #[test]
    fn test_expected_value_three_assets() {
        let test_portfolio = get_test_portfolio_with_three_assets();

        let logger = create_test_logger();
        assert_close!(
            0.285,
            expected_return(&test_portfolio, &logger),
            company::TOLERANCE
        );
    }

    #[test]
    fn test_all_outcomes_no_assets() {
        // Create an empty portfolio and attempt to calculate all outcomes, which fails
        let test_portfolio = Portfolio { companies: vec![] };
        let all_outcomes = all_outcomes(&test_portfolio).unwrap();

        assert_eq!(all_outcomes, vec![]);
    }

    #[test]
    fn test_all_outcomes_too_many_assets_and_scenarios() {
        // Create a portfolio with 16 companies, each with 2 scenarios
        let mut test_portfolio: Portfolio = Portfolio { companies: vec![] };
        for i in 0..16 {
            test_portfolio.companies.push(PortfolioCompany {
                company: Company {
                    name: format!("{i}"),
                    ticker: format!("{i}"),
                    description: format!("{i}"),
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
                fraction: 0.0625,
            });
        }

        // Should fail because there's more than 50000 outcomes
        let e = all_outcomes(&test_portfolio).err().unwrap();
        assert_eq!(e.code, "more-than-fifty-thousand-outcomes");
        assert!(e
            .message
            .contains("You have 65536 different outcomes for your portfolio."));
    }

    #[test]
    fn test_all_outcomes_three_assets() {
        let test_portfolio = get_test_portfolio_with_three_assets();
        let all_outcomes = all_outcomes(&test_portfolio).unwrap();

        assert_eq!(
            all_outcomes,
            vec![
                Outcome {
                    weighted_return: 1.0,
                    probability: 0.09,
                    company_returns: HashMap::from([
                        ("A".to_string(), 1.0),
                        ("B".to_string(), 1.0),
                        ("C".to_string(), 1.0),
                    ]),
                },
                Outcome {
                    weighted_return: 0.6,
                    probability: 0.09,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), 1.0),
                        ("C".to_string(), 1.0),
                    ]),
                },
                Outcome {
                    weighted_return: 0.4,
                    probability: 0.06,
                    company_returns: HashMap::from([
                        ("A".to_string(), 1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 1.0),
                    ]),
                },
                Outcome {
                    weighted_return: 0.0,
                    probability: 0.06,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 1.0),
                    ]),
                },
                Outcome {
                    weighted_return: 0.75,
                    probability: 0.09,
                    company_returns: HashMap::from([
                        ("A".to_string(), 1.0),
                        ("B".to_string(), 1.0),
                        ("C".to_string(), 0.5),
                    ]),
                },
                Outcome {
                    weighted_return: 0.35,
                    probability: 0.09,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), 1.0),
                        ("C".to_string(), 0.5),
                    ]),
                },
                Outcome {
                    weighted_return: 0.15,
                    probability: 0.06,
                    company_returns: HashMap::from([
                        ("A".to_string(), 1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 0.5),
                    ]),
                },
                Outcome {
                    weighted_return: -0.25,
                    probability: 0.06,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 0.5),
                    ]),
                },
                Outcome {
                    weighted_return: 0.5,
                    probability: 0.12,
                    company_returns: HashMap::from([
                        ("A".to_string(), 1.0),
                        ("B".to_string(), 1.0),
                        ("C".to_string(), 0.0),
                    ]),
                },
                Outcome {
                    weighted_return: 0.1,
                    probability: 0.12,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), 1.0),
                        ("C".to_string(), 0.0),
                    ]),
                },
                Outcome {
                    weighted_return: -0.1,
                    probability: 0.08,
                    company_returns: HashMap::from([
                        ("A".to_string(), 1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 0.0),
                    ]),
                },
                Outcome {
                    weighted_return: -0.5,
                    probability: 0.08,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 0.0),
                    ]),
                },
            ]
        )
    }

    #[test]
    fn test_worst_case_scenario() {
        let logger = create_test_logger();

        let test_portfolio = get_test_portfolio_with_three_assets();
        let all_outcomes = all_outcomes(&test_portfolio).unwrap();
        let worst_case = worst_case_outcome(&all_outcomes, &logger);

        assert_eq!(
            *worst_case,
            Outcome {
                weighted_return: -0.5,
                probability: 0.08,
                company_returns: HashMap::from([
                    ("A".to_string(), -1.0),
                    ("B".to_string(), -1.0),
                    ("C".to_string(), 0.0),
                ]),
            }
        )
    }

    #[test]
    fn test_cumulative_probability_of_loss() {
        let logger = create_test_logger();

        let test_portfolio = get_test_portfolio_with_three_assets();
        let all_outcomes = all_outcomes(&test_portfolio).unwrap();
        let cumulative_probability_of_loss = cumulative_probability_of_loss(&all_outcomes, &logger);

        assert_close!(0.22, cumulative_probability_of_loss, company::TOLERANCE);
    }
}
