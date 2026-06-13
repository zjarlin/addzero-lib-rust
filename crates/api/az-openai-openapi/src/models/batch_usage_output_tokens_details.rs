// Generated from OpenAPI spec. Do not edit by hand.
//! `BatchUsageOutputTokensDetails` DTO.

use serde::{Deserialize, Serialize};

/// A detailed breakdown of the output tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUsageOutputTokensDetails {
    /// The number of reasoning tokens.
    pub reasoning_tokens: i32,
}
