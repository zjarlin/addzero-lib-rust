// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatSessionRateLimits` DTO.

use serde::{Deserialize, Serialize};

/// Active per-minute request limit for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionRateLimits {
    /// Maximum allowed requests per one-minute window.
    pub max_requests_per_1_minute: i32,
}
