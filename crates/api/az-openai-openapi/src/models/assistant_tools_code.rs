// Generated from OpenAPI spec. Do not edit by hand.
//! `AssistantToolsCode` DTO.

use serde::{Deserialize, Serialize};

/// Code interpreter tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantToolsCode {
    /// The type of tool being defined: `code_interpreter`
    #[serde(rename = "type")]
    pub type_value: String,
}
