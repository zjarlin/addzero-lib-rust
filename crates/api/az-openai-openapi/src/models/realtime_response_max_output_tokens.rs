// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeResponseMaxOutputTokens` DTO.

use serde::{Deserialize, Serialize};

/// Maximum number of output tokens for a single assistant response, inclusive of tool calls, that was
/// used in this response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeResponseMaxOutputTokens {
    Integer(i32),
    Inf(String),
}
