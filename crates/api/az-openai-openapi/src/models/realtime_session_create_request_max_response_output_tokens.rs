// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeSessionCreateRequestMaxResponseOutputTokens` DTO.

use serde::{Deserialize, Serialize};

/// Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an
/// integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a
/// given model. Defaults to `inf`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateRequestMaxResponseOutputTokens {
    Integer(i32),
    Inf(String),
}
