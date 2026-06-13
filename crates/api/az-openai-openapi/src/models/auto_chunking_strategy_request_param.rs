// Generated from OpenAPI spec. Do not edit by hand.
//! `AutoChunkingStrategyRequestParam` DTO.

use serde::{Deserialize, Serialize};

/// The default strategy. This strategy currently uses a `max_chunk_size_tokens` of `800` and
/// `chunk_overlap_tokens` of `400`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoChunkingStrategyRequestParam {
    /// Always `auto`.
    #[serde(rename = "type")]
    pub type_value: String,
}
