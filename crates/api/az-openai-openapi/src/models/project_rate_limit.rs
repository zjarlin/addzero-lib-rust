// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectRateLimit` DTO.

use serde::{Deserialize, Serialize};

/// Represents a project rate limit config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRateLimit {
    /// The object type, which is always `project.rate_limit`
    pub object: String,
    /// The identifier, which can be referenced in API endpoints.
    pub id: String,
    /// The model this rate limit applies to.
    pub model: String,
    /// The maximum requests per minute.
    pub max_requests_per_1_minute: i32,
    /// The maximum tokens per minute.
    pub max_tokens_per_1_minute: i32,
    /// The maximum images per minute. Only present for relevant models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_images_per_1_minute: Option<i32>,
    /// The maximum audio megabytes per minute. Only present for relevant models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_audio_megabytes_per_1_minute: Option<i32>,
    /// The maximum requests per day. Only present for relevant models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_day: Option<i32>,
    /// The maximum batch input tokens per day. Only present for relevant models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_1_day_max_input_tokens: Option<i32>,
}
