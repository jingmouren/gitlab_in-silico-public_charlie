use crate::model::company::Company;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Portfolio has a list of portfolio companies
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Portfolio {
    pub companies: Vec<PortfolioCompany>,
}

/// Portfolio company represents a company with an associated allocation fraction
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PortfolioCompany {
    pub company: Company,
    pub fraction: f64,
}

/// Portfolio candidates has a list of companies
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PortfolioCandidates {
    pub companies: Vec<Company>,
}
