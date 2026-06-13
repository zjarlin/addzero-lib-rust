// Generated from OpenAPI spec. Do not edit by hand.
//! `UsageCompletionsResult` DTO.

use serde::{Deserialize, Serialize};

/// The aggregated completions usage details of the specific time bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCompletionsResult {
    pub object: String,
    /// The aggregated number of text input tokens used, including cached tokens. For customers subscribe to
    /// scale tier, this includes scale tier tokens.
    pub input_tokens: i32,
    /// The aggregated number of text input tokens that has been cached from previous requests. For
    /// customers subscribe to scale tier, this includes scale tier tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_cached_tokens: Option<i32>,
    /// The aggregated number of text output tokens used. For customers subscribe to scale tier, this
    /// includes scale tier tokens.
    pub output_tokens: i32,
    /// The aggregated number of audio input tokens used, including cached tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_tokens: Option<i32>,
    /// The aggregated number of audio output tokens used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio_tokens: Option<i32>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
}
