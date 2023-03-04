mod allocation;
mod analysis;
pub mod model;

use crate::allocation::kelly_criterion_allocate;
use crate::analysis::{all_outcomes, worst_case_outcome};
use crate::analysis::{cumulative_probability_of_loss, expected_return};
use crate::model::company::Company;

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
    let candidates: Vec<Company> = serde_yaml::from_str(yaml_string).unwrap();
    candidates.iter().for_each(|c| c.validate());
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

    let portfolio = kelly_criterion_allocate(filtered_candidates);

    portfolio.iter().for_each(|pc| {
        println!(
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
