// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionNamedToolChoiceCustom` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionNamedToolChoiceCustomCustom,
};

/// Specifies a tool the model should use. Use to force the model to call a specific custom tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionNamedToolChoiceCustom {
    /// For custom tool calling, the type is always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub custom: ChatCompletionNamedToolChoiceCustomCustom,
}
