// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ToolChoiceCustom` DTO.

use serde::{Deserialize, Serialize};

/// Use this option to force the model to call a specific custom tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceCustom {
    /// For custom tool calling, the type is always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the custom tool to call.
    pub name: String,
}
