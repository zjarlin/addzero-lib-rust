// Generated from OpenAPI spec. Do not edit by hand.
//! `RateLimitsParam` DTO.

use serde::{Deserialize, Serialize};

/// Controls request rate limits for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitsParam {
    /// Maximum number of requests allowed per minute for the session. Defaults to 10.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_minute: Option<i32>,
}
