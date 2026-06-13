// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateCompletionResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CompletionUsage,
    CreateCompletionResponseChoice,
};

/// Represents a completion response from the API. Note: both the streamed and non-streamed response
/// objects share the same shape (unlike the chat endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCompletionResponse {
    /// A unique identifier for the completion.
    pub id: String,
    /// The list of completion choices the model generated for the input prompt.
    pub choices: Vec<CreateCompletionResponseChoice>,
    /// The Unix timestamp (in seconds) of when the completion was created.
    pub created: i64,
    /// The model used for completion.
    pub model: String,
    /// This fingerprint represents the backend configuration that the model runs with. Can be used in
    /// conjunction with the `seed` request parameter to understand when backend changes have been made that
    /// might impact determinism.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    /// The object type, which is always "text_completion"
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,
}
