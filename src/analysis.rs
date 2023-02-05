use std::collections::HashMap;
use crate::model::company::Company;
use crate::Portfolio;

/// Helper struct for collecting outcomes and their associated probabilities
struct Outcome {
    portfolio_return: f64,
    probability: f64,
}

/// Returns all possible outcomes (expected portfolio return and associated probability)
fn all_outcomes(portfolio: &Portfolio) -> Vec<Outcome> {
    // Number of different outcomes is a product of number of all scenarios for all companies
    let n_outcomes = portfolio.keys().map(|c| c.scenarios.len()).product();

    if n_outcomes > 50000 {
        panic!("You have {} different outcomes for your portfolio. This software is designed for \
        a focused investment strategy, and it seems you have too many companies or too many \
        scenarios for companies.", n_outcomes)
    }

    // Mutable data that's populated/modified within the loop below
    // 1. Vectors for all outcomes
    let mut outcomes: Vec<Outcome> = Vec::with_capacity(n_outcomes);

    // 2. Helper map of current indices for scenarios of all companies
    let mut scenario_ids: HashMap<String, usize> = HashMap::new();
    portfolio.keys().for_each(|c| { scenario_ids.insert(c.ticker.clone(), 0); });

    while outcomes.len() != n_outcomes {
        // 1. Calculate the outcome by summing up scenarios for all companies
        let mut outcome = Outcome { portfolio_return: 0.0, probability: 0.0 };
        for (c, f) in *portfolio {
            let scenario_id = scenario_ids[c.ticker];
            let s = c.scenarios[scenario_id];

            outcome.portfolio_return +=
                f * s.probability * (s.intrinsic_value - c.market_cap) / c.market_cap;
            outcome.probability *= s.probability;
        }

        // 2. Append the calculated outcome to the list of outcomes
        outcomes.push(outcome);

        // 3. Increment a single index to prepare for the next iteration
        for c in *portfolio.keys() {
            if scenario_ids[c.ticker] + 1 == c.scenarios.len() {
                // We have exhausted the index for this company, set to zero and continue the loop
                scenario_ids.insert(c.ticker, 0);
                continue;
            } else {
                // Increment the first non-overflowing index and break out
                scenario_ids.insert(c.ticker, scenario_ids[c.ticker]) += 1;
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
        .map(|(c, f)| c.scenarios
            .iter()
            .map(|s| f * s.probability * (s.intrinsic_value - c.market_cap) / c.market_cap)
            .sum::<f64>()
        ).sum();

    println!("For every 1 dollar invested, we expect to end up with {} dollars", 1.0 + expected_return);

    expected_return
}

#[cfg(test)]
mod test {
    use approx::{__assert_approx, abs_diff_eq, relative_eq};
    use crate::model::company::Company;
    use crate::model::scenario::Scenario;
    use super::*;

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

        abs_diff_eq!(expected_return(&test_portfolio), 0.0);
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

        relative_eq!(expected_return(&test_portfolio), 0.6);
    }

    #[test]
    fn test_expected_value_three_assets() {
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
                        thesis: "Same as now".to_string(),
                        intrinsic_value: 1e8,
                        probability: 0.7,
                    },
                ],
            },
            0.5,
        );

        relative_eq!(expected_return(&test_portfolio), 0.21);
    }
}