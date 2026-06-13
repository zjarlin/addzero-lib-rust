// Generated from OpenAPI spec. Do not edit by hand.
//! `CompactResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ItemField,
    ResponseUsage,
};

/// The compacted response object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactResource {
    /// The unique identifier for the compacted response.
    pub id: String,
    /// The object type. Always `response.compaction`.
    pub object: String,
    /// The compacted list of output items.
    pub output: Vec<ItemField>,
    /// Unix timestamp (in seconds) when the compacted conversation was created.
    pub created_at: i64,
    /// Token accounting for the compaction pass, including cached, reasoning, and total tokens.
    pub usage: ResponseUsage,
}
