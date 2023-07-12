use crate::model::capital_loss::CapitalLoss;
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

/// Allocation input consists of a list of candidate companies and additional settings
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct AllocationInput {
    pub candidates: Vec<Company>,
    pub max_permanent_loss_of_capital: Option<CapitalLoss>,
}
