// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogRateLimitUpdatedChangesRequested` DTO.

use serde::{Deserialize, Serialize};

/// The payload used to update the rate limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRateLimitUpdatedChangesRequested {
    /// The maximum requests per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_minute: Option<i32>,
    /// The maximum tokens per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens_per_1_minute: Option<i32>,
    /// The maximum images per minute. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_images_per_1_minute: Option<i32>,
    /// The maximum audio megabytes per minute. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_audio_megabytes_per_1_minute: Option<i32>,
    /// The maximum requests per day. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_day: Option<i32>,
    /// The maximum batch input tokens per day. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_1_day_max_input_tokens: Option<i32>,
}
