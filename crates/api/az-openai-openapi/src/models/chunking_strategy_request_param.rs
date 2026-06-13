// Generated from OpenAPI spec. Do not edit by hand.
//! `ChunkingStrategyRequestParam` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// The chunking strategy used to chunk the file(s). If not set, will use the `auto` strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingStrategyRequestParam {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
