// Generated from OpenAPI spec. Do not edit by hand.
//! `CustomToolChatCompletionsCustom` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CustomToolChatCompletionsCustomFormat3,
};

/// Properties of the custom tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolChatCompletionsCustom {
    /// The name of the custom tool, used to identify it in tool calls.
    pub name: String,
    /// Optional description of the custom tool, used to provide more context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The input format for the custom tool. Default is unconstrained text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<CustomToolChatCompletionsCustomFormat3>,
}
