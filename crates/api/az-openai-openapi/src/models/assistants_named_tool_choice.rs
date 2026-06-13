// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AssistantsNamedToolChoice` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantsNamedToolChoiceFunction,
};

/// Specifies a tool the model should use. Use to force the model to call a specific tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantsNamedToolChoice {
    /// The type of the tool. If type is `function`, the function name must be set
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<AssistantsNamedToolChoiceFunction>,
}
