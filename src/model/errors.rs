use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Error with a message and a code. The code should be an internal identifier that indicates what
/// happened, while the message should be user-facing message that is supposed to help the user
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Clone, Debug)]
pub struct Error {
    pub code: String,
    pub message: String,
}

/// Same as the error, but represents a warning
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Clone, Debug)]
pub struct Warning {
    pub code: String,
    pub message: String,
}
