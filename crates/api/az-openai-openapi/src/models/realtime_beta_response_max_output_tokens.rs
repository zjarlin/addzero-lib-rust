// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaResponseMaxOutputTokens` DTO.

use serde::{Deserialize, Serialize};

/// Maximum number of output tokens for a single assistant response, inclusive of tool calls, that was
/// used in this response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeBetaResponseMaxOutputTokens {
    Integer(i32),
    Inf(String),
}
