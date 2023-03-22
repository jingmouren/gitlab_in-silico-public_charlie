use crate::model::company::Company;
use log::{error, info};
use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::http::{ContentType, Status};
use rocket::outcome::Outcome::{Failure, Success};
use rocket::{Data, Request};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Portfolio has a list of portfolio companies
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Portfolio {
    pub companies: Vec<PortfolioCompany>,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for Portfolio {
    type Error = String;

    async fn from_data(request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        // Ensure the content type is correct before opening the data.
        if request.content_type() != Some(&ContentType::JSON) {
            info!("Did not receive JSON, returning.");
            return Outcome::Forward(data);
        }

        // Read the data into a String, return 500 if it fails
        let string = match data.open(1024.kilobytes()).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Failure((Status::PayloadTooLarge, "Too large".to_string())),
            Err(e) => {
                error!(
                    "Unable to read data from the request. Request: {:?}",
                    request
                );
                return Failure((Status::InternalServerError, format!("{e}")));
            }
        };

        // Handle deserialization errors, return 400 if it fails
        let portfolio: Portfolio = match serde_json::from_str(&string) {
            Ok(r) => r,
            Err(e) => {
                info!(
                    "Did not manage to deserialize payload into portfolio. Error: {:?}",
                    e
                );
                return Failure((Status::BadRequest, format!("{e}")));
            }
        };

        Success(portfolio)
    }
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

#[rocket::async_trait]
impl<'r> FromData<'r> for PortfolioCandidates {
    type Error = String;

    async fn from_data(request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        // Ensure the content type is correct before opening the data.
        if request.content_type() != Some(&ContentType::JSON) {
            info!("Did not receive JSON, returning.");
            return Outcome::Forward(data);
        }

        // Read the data into a String, return 500 if it fails
        let string = match data.open(1024.kilobytes()).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Failure((Status::PayloadTooLarge, "Too large".to_string())),
            Err(e) => {
                error!(
                    "Unable to read data from the request. Request: {:?}",
                    request
                );
                return Failure((Status::InternalServerError, format!("{e}")));
            }
        };

        // Handle deserialization errors, return 400 if it fails
        let portfolio_candidates: PortfolioCandidates = match serde_json::from_str(&string) {
            Ok(r) => r,
            Err(e) => {
                info!(
                    "Did not manage to deserialize payload into candidates. Error: {:?}",
                    e
                );
                return Failure((Status::BadRequest, format!("{e}")));
            }
        };

        Success(portfolio_candidates)
    }
}
