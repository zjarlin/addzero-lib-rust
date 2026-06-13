// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventRateLimitsUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeServerEventRateLimitsUpdatedRateLimit,
};

/// Emitted at the beginning of a Response to indicate the updated rate limits. When a Response is
/// created some tokens will be "reserved" for the output tokens, the rate limits shown here reflect
/// that reservation, which is then adjusted accordingly once the Response is completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventRateLimitsUpdated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `rate_limits.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// List of rate limit information.
    pub rate_limits: Vec<RealtimeServerEventRateLimitsUpdatedRateLimit>,
}
