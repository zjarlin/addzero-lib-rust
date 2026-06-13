// Generated from OpenAPI spec. Do not edit by hand.
//! `VectorStoreFileObjectChunkingStrategy` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OtherChunkingStrategyResponseParam,
    StaticChunkingStrategyResponseParam,
};

/// The strategy used to chunk the file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VectorStoreFileObjectChunkingStrategy {
    StaticChunkingStrategyResponseParam(StaticChunkingStrategyResponseParam),
    OtherChunkingStrategyResponseParam(OtherChunkingStrategyResponseParam),
}
