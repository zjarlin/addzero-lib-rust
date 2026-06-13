// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionToolParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EmptyModelParam,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionToolParam {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<EmptyModelParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(rename = "type")]
    pub type_value: String,
    /// Whether this function should be deferred and discovered via tool search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
}
