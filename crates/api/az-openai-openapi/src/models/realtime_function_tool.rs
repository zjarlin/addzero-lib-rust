// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeFunctionTool` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// Function tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeFunctionTool {
    /// The type of the tool, i.e. `function`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The name of the function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The description of the function, including guidance on when and how to call it, and guidance about
    /// what to tell the user when calling (if anything).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Parameters of the function in JSON Schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<OpenAiJsonObject>,
}
