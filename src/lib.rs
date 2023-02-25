mod allocation;
mod analysis;
pub mod model;

use crate::allocation::kelly_criterion_allocate;
use crate::analysis::{all_outcomes, worst_case_outcome};
use crate::analysis::{cumulative_probability_of_loss, expected_return};
use crate::model::company::Company;
use std::collections::HashMap;

/// Portfolio is a map of companies with associated fractions/allocations (e.g. company ABC is 20%
/// of the portfolio)
pub type Portfolio = HashMap<Company, f64>;

/// Creates a vector of candidate companies from YAML
pub fn create_candidates(yaml_string: &str) -> Vec<Company> {
    let candidates: Vec<Company> = serde_yaml::from_str(yaml_string).unwrap();
    candidates
}

/// Calculates optimal allocation for each candidate company
pub fn allocate(mut candidates: Vec<Company>) -> Portfolio {
    // Retain only the candidates that have positive expected value. This would otherwise likely
    // lead to negative fractions (which implies shorting). Note that I said "likely" because I'm
    // not 100% sure, but just have a feeling.
    candidates.retain(|c| {
        c.scenarios
            .iter()
            .map(|s| s.probability * (s.intrinsic_value - c.market_cap) / c.market_cap)
            .sum::<f64>()
            > 0.0
    });
    kelly_criterion_allocate(candidates)
}

/// Prints useful information about the portfolio
pub fn analyse(portfolio: &Portfolio) {
    expected_return(portfolio);

    let all_outcomes = all_outcomes(portfolio);
    worst_case_outcome(&all_outcomes);
    cumulative_probability_of_loss(&all_outcomes);
}
