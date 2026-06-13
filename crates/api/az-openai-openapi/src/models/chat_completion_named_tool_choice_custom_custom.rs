// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionNamedToolChoiceCustomCustom` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionNamedToolChoiceCustomCustom {
    /// The name of the custom tool to call.
    pub name: String,
}
