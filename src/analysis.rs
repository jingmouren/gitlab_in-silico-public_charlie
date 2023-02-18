use crate::model::company::{Company, Ticker};
use crate::Portfolio;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use std::collections::HashMap;

/// An outcome consists of its probability and portfolio return
#[derive(Debug)]
pub struct Outcome {
    pub weighted_return: f64,
    pub probability: f64,
    pub company_returns: HashMap<Ticker, f64>,
}

/// Returns all possible outcomes (expected portfolio return and associated probability)
pub fn all_outcomes(portfolio: &Portfolio) -> Vec<Outcome> {
    // Number of different outcomes is a product of number of all scenarios for all companies
    let n_outcomes = if !portfolio.is_empty() {
        portfolio.keys().map(|c| c.scenarios.len()).product()
    } else {
        0
    };

    if n_outcomes > 50000 {
        panic!(
            "You have {n_outcomes} different outcomes for your portfolio. This software is \
            designed for a focused investment strategy, and it seems you have too many companies \
            or too many scenarios for companies.",
        )
    }

    // Collect relevant portfolio data into a sorted Vec to have reproducibility in tests
    let companies_and_fractions: Vec<(&Company, f64)> = portfolio
        .iter()
        .map(|(c, f)| (c, *f))
        .sorted_by(|(c1, _), (c2, _)| c1.ticker.cmp(&c2.ticker))
        .collect();

    // Mutable data that's populated/modified within the loop below
    // 1. Vectors for all outcomes
    let mut outcomes: Vec<Outcome> = Vec::with_capacity(n_outcomes);

    // 2. Helper vectors keeping track of current indices for scenarios of all companies
    let mut scenario_indices: Vec<usize> = vec![0; companies_and_fractions.len()];
    let n_scenarios: Vec<usize> = companies_and_fractions
        .iter()
        .map(|(c, _)| c.scenarios.len())
        .collect();

    // Start filling in outcomes until all are collected
    while outcomes.len() != n_outcomes {
        // 1. Calculate the outcome by summing up scenarios for all companies
        // Note: Probability is initialized with 1.0 since we multiply to get joint probability
        let mut outcome = Outcome {
            weighted_return: 0.0,
            probability: 1.0,
            company_returns: HashMap::with_capacity(portfolio.len()),
        };

        companies_and_fractions
            .iter()
            .enumerate()
            .for_each(|(ticker_id, (c, f))| {
                let scenario_id = scenario_indices[ticker_id];
                let s = &c.scenarios[scenario_id];

                let company_return = (s.intrinsic_value - c.market_cap) / c.market_cap;
                outcome.weighted_return += f * company_return;
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

    outcomes
}

/// Calculates expected return of a portfolio
pub fn expected_return(portfolio: &Portfolio) -> f64 {
    let expected_return: f64 = portfolio
        .iter()
        .map(|(c, f)| {
            c.scenarios
                .iter()
                .map(|s| f * s.probability * (s.intrinsic_value - c.market_cap) / c.market_cap)
                .sum::<f64>()
        })
        .sum();

    println!(
        "For every 1 dollar invested, we expect to end up with {} dollars",
        1.0 + expected_return
    );

    expected_return
}

/// Finds an outcome with maximum loss of capital and reports its probability
pub fn worst_case_outcome(outcomes: &[Outcome]) -> &Outcome {
    let worst_case_outcome = outcomes
        .iter()
        .min_by_key(|o| OrderedFloat(o.weighted_return))
        .unwrap(); // TODO: Handle errors

    println!(
        "Worst case outcome implies permanent loss of {}% of invested assets with probability {}%",
        100.0 * worst_case_outcome.weighted_return,
        100.0 * worst_case_outcome.probability
    );

    worst_case_outcome
}

/// Calculates the cumulative probability of losing money
pub fn cumulative_probability_of_loss(outcomes: &[Outcome]) -> f64 {
    let cumulative_probability_of_loss = outcomes
        .iter()
        .filter(|o| o.weighted_return < 0.0)
        .map(|o| o.probability)
        .sum();

    println!("Cumulative probability of loss of capital is {cumulative_probability_of_loss}");

    cumulative_probability_of_loss
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::company;
    use crate::model::company::Company;
    use crate::model::scenario::Scenario;
    use crate::HashMap;

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
    fn get_test_portfolio_with_three_assets() -> HashMap<Company, f64> {
        let mut test_portfolio = Portfolio::new();

        // Unbiased coin flip with 20% allocation
        test_portfolio.insert(
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
            0.2,
        );

        // Biased coin flip with 30% allocation
        test_portfolio.insert(
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
            0.3,
        );

        // Something with only upside 50% allocation
        test_portfolio.insert(
            Company {
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
            0.5,
        );

        test_portfolio
    }

    #[test]
    fn test_expected_value_single_fair_coin_flip() {
        let mut test_portfolio = Portfolio::new();

        // Unbiased coin flip with 100% allocation
        test_portfolio.insert(
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
            1.0,
        );

        assert!(expected_return(&test_portfolio) < company::TOLERANCE);
    }

    #[test]
    fn test_expected_value_single_biased_coin_flip() {
        let mut test_portfolio = Portfolio::new();

        // Biased coin flip with 100% allocation
        test_portfolio.insert(
            Company {
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
            1.0,
        );

        assert!((expected_return(&test_portfolio) - 0.6).abs() < company::TOLERANCE);
    }

    #[test]
    fn test_expected_value_three_assets() {
        let test_portfolio = get_test_portfolio_with_three_assets();

        assert!((expected_return(&test_portfolio) - 0.285).abs() < company::TOLERANCE);
    }

    #[test]
    fn test_all_outcomes_no_assets() {
        // Create an empty portfolio and attempt to calculate all outcomes, which fails
        let test_portfolio = Portfolio::new();
        let all_outcomes = all_outcomes(&test_portfolio);

        assert_eq!(all_outcomes, vec![]);
    }

    #[test]
    #[should_panic(expected = "You have 65536 different outcomes for your portfolio.")]
    fn test_all_outcomes_too_many_assets_and_scenarios() {
        // Create a portfolio with 16 companies, each with 2 scenarios
        let mut test_portfolio = Portfolio::new();
        for i in 0..16 {
            test_portfolio.insert(
                Company {
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
                0.0625,
            );
        }

        // Should fail because there's more than 50000 outcomes
        all_outcomes(&test_portfolio);
    }

    #[test]
    fn test_all_outcomes_three_assets() {
        let test_portfolio = get_test_portfolio_with_three_assets();
        let all_outcomes = all_outcomes(&test_portfolio);

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
                    ])
                },
                Outcome {
                    weighted_return: 0.6,
                    probability: 0.09,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), 1.0),
                        ("C".to_string(), 1.0),
                    ])
                },
                Outcome {
                    weighted_return: 0.4,
                    probability: 0.06,
                    company_returns: HashMap::from([
                        ("A".to_string(), 1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 1.0),
                    ])
                },
                Outcome {
                    weighted_return: 0.0,
                    probability: 0.06,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 1.0),
                    ])
                },
                Outcome {
                    weighted_return: 0.75,
                    probability: 0.09,
                    company_returns: HashMap::from([
                        ("A".to_string(), 1.0),
                        ("B".to_string(), 1.0),
                        ("C".to_string(), 0.5),
                    ])
                },
                Outcome {
                    weighted_return: 0.35,
                    probability: 0.09,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), 1.0),
                        ("C".to_string(), 0.5),
                    ])
                },
                Outcome {
                    weighted_return: 0.15,
                    probability: 0.06,
                    company_returns: HashMap::from([
                        ("A".to_string(), 1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 0.5),
                    ])
                },
                Outcome {
                    weighted_return: -0.25,
                    probability: 0.06,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 0.5),
                    ])
                },
                Outcome {
                    weighted_return: 0.5,
                    probability: 0.12,
                    company_returns: HashMap::from([
                        ("A".to_string(), 1.0),
                        ("B".to_string(), 1.0),
                        ("C".to_string(), 0.0),
                    ])
                },
                Outcome {
                    weighted_return: 0.1,
                    probability: 0.12,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), 1.0),
                        ("C".to_string(), 0.0),
                    ])
                },
                Outcome {
                    weighted_return: -0.1,
                    probability: 0.08,
                    company_returns: HashMap::from([
                        ("A".to_string(), 1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 0.0),
                    ])
                },
                Outcome {
                    weighted_return: -0.5,
                    probability: 0.08,
                    company_returns: HashMap::from([
                        ("A".to_string(), -1.0),
                        ("B".to_string(), -1.0),
                        ("C".to_string(), 0.0),
                    ])
                },
            ]
        )
    }

    #[test]
    fn test_worst_case_scenario() {
        let test_portfolio = get_test_portfolio_with_three_assets();
        let all_outcomes = all_outcomes(&test_portfolio);
        let worst_case = worst_case_outcome(&all_outcomes);

        assert_eq!(
            *worst_case,
            Outcome {
                weighted_return: -0.5,
                probability: 0.08,
                company_returns: HashMap::from([
                    ("A".to_string(), -1.0),
                    ("B".to_string(), -1.0),
                    ("C".to_string(), 0.0),
                ])
            }
        )
    }

    #[test]
    fn test_cumulative_probability_of_loss() {
        let test_portfolio = get_test_portfolio_with_three_assets();
        let all_outcomes = all_outcomes(&test_portfolio);
        let cumulative_probability_of_loss = cumulative_probability_of_loss(&all_outcomes);

        assert!((cumulative_probability_of_loss - 0.22).abs() < company::TOLERANCE);
    }
}
