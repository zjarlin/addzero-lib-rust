// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateChatCompletionStreamResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CompletionUsage,
    CreateChatCompletionStreamResponseChoice,
    ServiceTier,
};

/// Represents a streamed chunk of a chat completion response returned by the model, based on the
/// provided input. [Learn more](/docs/guides/streaming-responses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatCompletionStreamResponse {
    /// A unique identifier for the chat completion. Each chunk has the same ID.
    pub id: String,
    /// A list of chat completion choices. Can contain more than one elements if `n` is greater than 1. Can
    /// also be empty for the last chunk if you set `stream_options: {"include_usage": true}`.
    pub choices: Vec<CreateChatCompletionStreamResponseChoice>,
    /// The Unix timestamp (in seconds) of when the chat completion was created. Each chunk has the same
    /// timestamp.
    pub created: i64,
    /// The model to generate the completion.
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    /// This fingerprint represents the backend configuration that the model runs with. Can be used in
    /// conjunction with the `seed` request parameter to understand when backend changes have been made that
    /// might impact determinism.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    /// The object type, which is always `chat.completion.chunk`.
    pub object: String,
    /// An optional field that will only be present when you set `stream_options: {"include_usage": true}`
    /// in your request. When present, it contains a null value **except for the last chunk** which contains
    /// the token usage statistics for the entire request. **NOTE:** If the stream is interrupted or
    /// cancelled, you may not receive the final usage chunk which contains the total token usage for the
    /// request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,
}
