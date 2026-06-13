// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateChatCompletionResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CompletionUsage,
    CreateChatCompletionResponseChoice,
    ServiceTier,
};

/// Represents a chat completion response returned by model, based on the provided input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatCompletionResponse {
    /// A unique identifier for the chat completion.
    pub id: String,
    /// A list of chat completion choices. Can be more than one if `n` is greater than 1.
    pub choices: Vec<CreateChatCompletionResponseChoice>,
    /// The Unix timestamp (in seconds) of when the chat completion was created.
    pub created: i64,
    /// The model used for the chat completion.
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    /// This fingerprint represents the backend configuration that the model runs with. Can be used in
    /// conjunction with the `seed` request parameter to understand when backend changes have been made that
    /// might impact determinism.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    /// The object type, which is always `chat.completion`.
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,
}
