// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateRunRequestTruncationStrategy` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRunRequestTruncationStrategy {
    /// The truncation strategy to use for the thread. The default is `auto`. If set to `last_messages`, the
    /// thread will be truncated to the n most recent messages in the thread. When set to `auto`, messages
    /// in the middle of the thread will be dropped to fit the context length of the model,
    /// `max_prompt_tokens`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_messages: Option<i32>,
}
