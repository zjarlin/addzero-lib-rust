// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `StaticChunkingStrategyResponseParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    StaticChunkingStrategy,
};

/// Static Chunking Strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticChunkingStrategyResponseParam {
    /// Always `static`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(rename = "static")]
    pub static_value: StaticChunkingStrategy,
}
