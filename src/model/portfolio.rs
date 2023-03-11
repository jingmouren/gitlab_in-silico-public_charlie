use crate::model::company::Company;
use log::{error, info};
use rocket::data::{FromDataSimple, Outcome};
use rocket::http::{ContentType, Status};
use rocket::Outcome::{Failure, Success};
use rocket::{Data, Request};
use serde::{Deserialize, Serialize};
use std::io::Read;

/// Portfolio has a list of portfolio companies
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Portfolio {
    pub portfolio_companies: Vec<PortfolioCompany>,
}

/// Portfolio company represents a company with an associated allocation fraction
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortfolioCompany {
    pub company: Company,
    pub fraction: f64,
}

/// Portfolio candidates has a list of companies
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortfolioCandidates {
    pub companies: Vec<Company>,
}

impl FromDataSimple for PortfolioCandidates {
    type Error = String;

    fn from_data(request: &Request, data: Data) -> Outcome<Self, String> {
        // Ensure the content type is correct before opening the data.
        if request.content_type() != Some(&ContentType::JSON) {
            info!("Did not receive JSON, returning.");
            return Outcome::Forward(data);
        }

        // Read the data into a String, return 500 if it fails
        let mut json_string = String::new();
        if let Err(e) = data
            .open()
            .take(1024 * 1024)
            .read_to_string(&mut json_string)
        {
            error!(
                "Unable to read data from the request. Request: {:?}",
                request
            );
            return Failure((Status::InternalServerError, format!("{:?}", e)));
        }

        // Handle deserialization errors, return 400 if it fails
        let portfolio_candidates: PortfolioCandidates = match serde_json::from_str(&json_string) {
            Ok(r) => r,
            Err(e) => {
                info!(
                    "Did not manage to deserialize payload into candidates. Error: {:?}",
                    e
                );
                return Failure((Status::BadRequest, format!("{:?}", e)));
            }
        };

        Success(portfolio_candidates)
    }
}
