mod allocation;
mod analysis;
mod model;

use crate::analysis::analysis::{all_outcomes, worst_case_outcome};
use crate::analysis::analysis::{cumulative_probability_of_loss, expected_return};
use crate::model::company::Company;
use std::collections::HashMap;

/// Portfolio is a map of companies with associated fractions/allocations (e.g. company ABC is 20%
/// of the portfolio)
pub type Portfolio = HashMap<Company, f64>;

/// (Portfolio) Candidates is a vector of companies under consideration for investment
pub type Candidates = Vec<Company>;

/// Creates a vector of candidate companies from YAML
pub fn create_candidates(yaml_string: &str) -> Candidates {
    // TODO: Recoverable error
    let candidates: Candidates = serde_yaml::from_str(yaml_string).unwrap();
    candidates
}

/// Prints useful information about the portfolio
pub fn analyse(portfolio: &Portfolio) {
    expected_return(portfolio);

    let all_outcomes = all_outcomes(portfolio);
    worst_case_outcome(&all_outcomes);
    cumulative_probability_of_loss(&all_outcomes);
}

/// Calculates optimal allocation for each candidate company
pub fn allocate(_candidates: &Candidates) -> Portfolio {
    todo!("Kelly formula allocation")
}
