// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CustomToolChatCompletions` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CustomToolChatCompletionsCustom,
};

/// A custom tool that processes input using a specified format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolChatCompletions {
    /// The type of the custom tool. Always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Properties of the custom tool.
    pub custom: CustomToolChatCompletionsCustom,
}
