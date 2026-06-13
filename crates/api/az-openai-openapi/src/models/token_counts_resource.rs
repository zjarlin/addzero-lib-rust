// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `TokenCountsResource` DTO.

use serde::{Deserialize, Serialize};

/// Token counts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCountsResource {
    pub object: String,
    pub input_tokens: i32,
}
