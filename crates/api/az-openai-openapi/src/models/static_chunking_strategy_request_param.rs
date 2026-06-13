// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `StaticChunkingStrategyRequestParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    StaticChunkingStrategy,
};

/// Customize your own chunking strategy by setting chunk size and chunk overlap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticChunkingStrategyRequestParam {
    /// Always `static`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(rename = "static")]
    pub static_value: StaticChunkingStrategy,
}
