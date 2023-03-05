mod allocation;
mod analysis;
pub mod model;
pub mod validation;

use crate::allocation::{kelly_allocate, MAX_ITER};
use crate::analysis::{all_outcomes, worst_case_outcome};
use crate::analysis::{cumulative_probability_of_loss, expected_return};
use crate::model::company::Company;
use crate::validation::result::ValidationResult;
use crate::validation::validate::Validate;
use log::info;
use std::collections::HashSet;

/// Portfolio is a vector of PortfolioCompany objects
pub type Portfolio = Vec<PortfolioCompany>;

/// Portfolio company is a company with an associated fraction
#[derive(Debug)]
pub struct PortfolioCompany {
    pub company: Company,
    pub fraction: f64,
}

/// Creates a vector of candidate companies from YAML
pub fn create_candidates(yaml_string: &str) -> Vec<Company> {
    // Deserialize candidates from yaml (TODO: Missing error handling)
    let candidates: Vec<Company> = serde_yaml::from_str(yaml_string).unwrap();

    // Collect all validation errors
    let mut all_validation_errors: HashSet<ValidationResult> = HashSet::new();
    candidates
        .iter()
        .for_each(|c| all_validation_errors.extend(c.validate()));

    // Panic at the moment: TODO: Error handling
    if all_validation_errors
        .iter()
        .any(|vr| *vr != ValidationResult::OK)
    {
        panic!("Found validation errors: {all_validation_errors:?}");
    }

    candidates
}

/// Calculates optimal allocation for each candidate company
pub fn allocate(candidates: Vec<Company>) -> Portfolio {
    // Retain only the candidates that have positive expected value. This would otherwise likely
    // lead to negative fractions (which implies shorting). Note that I said "likely" because I'm
    // not 100% sure, but just have a feeling.
    let filtered_candidates = candidates
        .iter()
        .cloned()
        .filter(|c| {
            c.scenarios
                .iter()
                .map(|s| s.probability * (s.intrinsic_value - c.market_cap) / c.market_cap)
                .sum::<f64>()
                > 0.0
        })
        .collect();

    // TODO:
    //  1. Add info statement for filtered candidates
    //  2. Filter also the "perfect" without any downside

    let portfolio = kelly_allocate(filtered_candidates, MAX_ITER);

    portfolio.iter().for_each(|pc| {
        info!(
            "Company: {}, fraction: {:.1}%",
            pc.company.name,
            100.0 * pc.fraction
        )
    });

    portfolio
}

/// Calculates and prints useful information about the portfolio
pub fn analyse(portfolio: &Portfolio) {
    expected_return(portfolio);

    let all_outcomes = all_outcomes(portfolio);
    worst_case_outcome(&all_outcomes);
    cumulative_probability_of_loss(&all_outcomes);
}
