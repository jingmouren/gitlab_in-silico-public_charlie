use std::collections::HashMap;
use crate::company::Company;

mod scenario;
mod company;

/// Portfolio is a map of companies with associated fractions/allocations (e.g. company ABC is 20%
/// of the portfolio)
pub type Portfolio = HashMap<Company, f64>;

/// (Portfolio) Candidates is a vector of companies under consideration for investment
pub type Candidates = Vec<Company>;

/// Creates a vector of candidate companies from YAML
pub fn create_candidates(yaml_string: &String) -> Candidates {
    // TODO: Recoverable error
    let candidates: Candidates = serde_yaml::from_str(&yaml_string).unwrap();
    return candidates
}

/// Calculates optimal allocation for each candidate company
pub fn allocate(candidates: &Candidates) -> Portfolio {
    // TODO: Implement
    panic!("Not implemented. Candidates are: {:?}", candidates);
}