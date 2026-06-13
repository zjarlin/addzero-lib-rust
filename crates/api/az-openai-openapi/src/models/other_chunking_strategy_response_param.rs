// Generated from OpenAPI spec. Do not edit by hand.
//! `OtherChunkingStrategyResponseParam` DTO.

use serde::{Deserialize, Serialize};

/// This is returned when the chunking strategy is unknown. Typically, this is because the file was
/// indexed before the `chunking_strategy` concept was introduced in the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherChunkingStrategyResponseParam {
    /// Always `other`.
    #[serde(rename = "type")]
    pub type_value: String,
}
