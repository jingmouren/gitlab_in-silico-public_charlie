use crate::model::company::Company;
use crate::Portfolio;
use itertools::Itertools;
use std::collections::HashMap;

/// Helper struct for collecting outcomes and their associated probabilities
#[derive(Debug)]
struct Outcome {
    portfolio_return: f64,
    probability: f64,
}

/// Returns all possible outcomes (expected portfolio return and associated probability)
fn all_outcomes(portfolio: &Portfolio) -> Vec<Outcome> {
    // Number of different outcomes is a product of number of all scenarios for all companies
    let n_outcomes = portfolio.keys().map(|c| c.scenarios.len()).product();

    if n_outcomes > 50000 {
        panic!(
            "You have {} different outcomes for your portfolio. This software is designed for \
        a focused investment strategy, and it seems you have too many companies or too many \
        scenarios for companies.",
            n_outcomes
        )
    }

    // Collect relevant portfolio data into a sorted Vec to have reproducibility in tests
    // Note: Items in the vector are reference counted Company instances so that we don't make
    // unnecessary copies
    let tickers_and_fractions: Vec<(&Company, f64)> = portfolio
        .iter()
        .map(|(c, f)| (c, f.clone()))
        .sorted_by(|(c1, _), (c2, _)| c1.ticker.cmp(&c2.ticker))
        .collect();

    // Mutable data that's populated/modified within the loop below
    // 1. Vectors for all outcomes
    let mut outcomes: Vec<Outcome> = Vec::with_capacity(n_outcomes);

    // 2. Helper vectors keeping track of current indices for scenarios of all companies
    let mut scenario_indices: Vec<usize> = vec![0; tickers_and_fractions.len()];
    let n_scenarios: Vec<usize> = portfolio.keys().map(|c| c.scenarios.len()).collect();

    // Start filling in outcomes until all are collected
    while outcomes.len() != n_outcomes {
        // 1. Calculate the outcome by summing up scenarios for all companies
        let mut outcome = Outcome {
            portfolio_return: 0.0,
            probability: 1.0,
        };
        tickers_and_fractions
            .iter()
            .enumerate()
            .for_each(|(ticker_id, (c, f))| {
                let scenario_id = scenario_indices[ticker_id];
                let s = &c.scenarios[scenario_id];

                outcome.portfolio_return +=
                    f * s.probability * (s.intrinsic_value - c.market_cap) / c.market_cap;
                outcome.probability *= s.probability;
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::company;
    use crate::model::company::Company;
    use crate::model::scenario::Scenario;

    impl PartialEq<Self> for Outcome {
        fn eq(&self, other: &Self) -> bool {
            ((self.portfolio_return - other.portfolio_return).abs() < company::TOLERANCE)
                && ((self.probability - other.probability).abs() < company::TOLERANCE)
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

        assert!(expected_return(&test_portfolio) < 1e-10);
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

        assert!((expected_return(&test_portfolio) - 0.6).abs() < 1e-10);
    }

    #[test]
    fn test_expected_value_three_assets() {
        let test_portfolio = get_test_portfolio_with_three_assets();
        assert!((expected_return(&test_portfolio) - 0.285).abs() < 1e-10);
    }

    #[test]
    fn test_all_outcomes() {
        let test_portfolio = get_test_portfolio_with_three_assets();
        let all_outcomes = all_outcomes(&test_portfolio);

        // TODO: Undefined behaviour in test: it occasionally fails

        assert_eq!(
            all_outcomes,
            vec![
                Outcome {
                    portfolio_return: 0.43,
                    probability: 0.09
                },
                Outcome {
                    portfolio_return: 0.23,
                    probability: 0.09
                },
                Outcome {
                    portfolio_return: 0.13,
                    probability: 0.06
                },
                Outcome {
                    portfolio_return: -0.07,
                    probability: 0.06
                },
                Outcome {
                    portfolio_return: 0.355,
                    probability: 0.09
                },
                Outcome {
                    portfolio_return: 0.155,
                    probability: 0.09
                },
                Outcome {
                    portfolio_return: 0.055,
                    probability: 0.06
                },
                Outcome {
                    portfolio_return: -0.145,
                    probability: 0.06
                },
                Outcome {
                    portfolio_return: 0.28,
                    probability: 0.12
                },
                Outcome {
                    portfolio_return: 0.08,
                    probability: 0.12
                },
                Outcome {
                    portfolio_return: -0.02,
                    probability: 0.08
                },
                Outcome {
                    portfolio_return: -0.22,
                    probability: 0.08
                },
            ]
        )
    }
}
