// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionTool` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonValue,
};

/// Defines a function in your own code the model can choose to call. Learn more about [function
/// calling](https://platform.openai.com/docs/guides/function-calling).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionTool {
    /// The type of the function tool. Always `function`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the function to call.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<std::collections::BTreeMap<String, OpenAiJsonValue>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    /// Whether this function is deferred and loaded via tool search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
}
