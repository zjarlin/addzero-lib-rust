// Generated from OpenAPI spec. Do not edit by hand.
//! `UsageEmbeddingsResult` DTO.

use serde::{Deserialize, Serialize};

/// The aggregated embeddings usage details of the specific time bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEmbeddingsResult {
    pub object: String,
    /// The aggregated number of input tokens used.
    pub input_tokens: i32,
    /// The count of requests made to the model.
    pub num_model_requests: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}
