use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};

use crate::scenario::Scenario;

/// Tolerance when validating that all probabilities across scenarios sum up to 1
const PROBABILITY_TOLERANCE: f64 = 1e-10;

/// A company with some basic information relevant for investment and a set of possible scenarios
#[derive(Serialize, Deserialize, Debug)]
pub struct Company {
    name: String,
    ticker: String,
    description: String,
    market_cap: f64,
    scenarios: HashSet<Scenario>,
}

/// Two companies are considered equal if their ticker symbols are equal. This is done in order to
/// handle possibly dually listed shares where some arbitrage may be present (i.e. different market
/// caps on different stock exchanges).
impl PartialEq<Self> for Company {
    fn eq(&self, other: &Self) -> bool {
        self.ticker == other.ticker
    }
}

impl Eq for Company {}

/// Hash key based on the ticker symbol
impl Hash for Company {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.ticker.hash(hasher)
    }
}

impl Company {
    /// Create a new instance of Company that does validation after initialization
    pub fn new(
        name: String,
        ticker: String,
        description: String,
        market_cap: f64,
        scenarios: Vec<Scenario>,
    ) -> Company {
        // Fail if we have duplicate scenarios
        if scenarios.len() != HashSet::from_iter(scenarios).len() {
            panic!("Having scenarios with the same thesis is not allowed. Scenarios are: {:?}", scenarios.map(|scenario| ", {}", scenario.thesis))
        }

        let company = Company {
            name,
            ticker,
            description,
            market_cap,
            scenarios,
        };

        company.validate();

        return company;
    }

    /// Does all validations. Used after construction
    fn validate(&self) {
        self.validate_all_scenarios_unique();
        self.validate_at_least_one_scenario();
        self.validate_probabilities_sum_up_to_one();
    }

    /// Panics if we don't have at least one scenario
    /// TODO: Convert panics to recoverable errors that can be handled
    fn validate_at_least_one_scenario(&self) {
        if self.scenarios.is_empty() {
            panic!(
                "No scenarios found for {name} with ticker {ticker}.",
                name = self.name, ticker = self.ticker
            )
        }
    }

    /// Panics if all probabilities across all scenarios don't sum up close to 1
    /// TODO: Convert panics to recoverable errors that can be handled
    fn validate_probabilities_sum_up_to_one(&self) {
        let sum: f64 = self.scenarios.iter().map(|scenario| scenario.probability).sum();
        if (sum - 1.0).abs() < PROBABILITY_TOLERANCE {
            panic!("Probabilities of all scenarios do not sum up to 1. Sum = {}.", sum)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_probability_tolerance_doesnt_change() {
        assert_eq!(PROBABILITY_TOLERANCE, 1e-10)
    }
}