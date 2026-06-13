// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaResponseCreateParamsTool` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaResponseCreateParamsTool {
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
