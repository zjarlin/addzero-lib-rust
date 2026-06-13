// Generated from OpenAPI spec. Do not edit by hand.
//! `ThreadObjectToolResourcesFileSearch` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadObjectToolResourcesFileSearch {
    /// The [vector store](/docs/api-reference/vector-stores/object) attached to this thread. There can be a
    /// maximum of 1 vector store attached to the thread.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_store_ids: Option<Vec<String>>,
}
