// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AssistantToolsFileSearchTypeOnly` DTO.

use serde::{Deserialize, Serialize};

/// FileSearch tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantToolsFileSearchTypeOnly {
    /// The type of tool being defined: `file_search`
    #[serde(rename = "type")]
    pub type_value: String,
}
