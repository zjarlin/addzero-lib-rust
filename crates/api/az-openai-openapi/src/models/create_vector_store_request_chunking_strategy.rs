// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateVectorStoreRequestChunkingStrategy` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AutoChunkingStrategyRequestParam,
    StaticChunkingStrategyRequestParam,
};

/// The chunking strategy used to chunk the file(s). If not set, will use the `auto` strategy. Only
/// applicable if `file_ids` is non-empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateVectorStoreRequestChunkingStrategy {
    AutoChunkingStrategyRequestParam(AutoChunkingStrategyRequestParam),
    StaticChunkingStrategyRequestParam(StaticChunkingStrategyRequestParam),
}
