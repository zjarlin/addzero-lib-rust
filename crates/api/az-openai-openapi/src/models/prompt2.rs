// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `Prompt2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponsePromptVariables,
};

/// Reference to a prompt template and its variables. [Learn more](/docs/guides/text?api-
/// mode=responses#reusable-prompts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt2 {
    /// The unique identifier of the prompt template to use.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<ResponsePromptVariables>,
}
