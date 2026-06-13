// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventRateLimitsUpdatedRateLimit` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventRateLimitsUpdatedRateLimit {
    /// The name of the rate limit (`requests`, `tokens`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The maximum allowed value for the rate limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// The remaining value before the limit is reached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining: Option<i32>,
    /// Seconds until the rate limit resets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_seconds: Option<f64>,
}
