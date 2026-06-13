// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FunctionParameters,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionObject {
    /// A description of what the function does, used by the model to choose when and how to call the
    /// function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The name of the function to be called. Must be a-z, A-Z, 0-9, or contain underscores and dashes,
    /// with a maximum length of 64.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<FunctionParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}
