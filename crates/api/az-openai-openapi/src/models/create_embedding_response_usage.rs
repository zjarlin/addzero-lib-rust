// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEmbeddingResponseUsage` DTO.

use serde::{Deserialize, Serialize};

/// The usage information for the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEmbeddingResponseUsage {
    /// The number of tokens used by the prompt.
    pub prompt_tokens: i32,
    /// The total number of tokens used by the request.
    pub total_tokens: i32,
}
