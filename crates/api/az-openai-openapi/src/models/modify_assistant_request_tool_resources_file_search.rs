// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ModifyAssistantRequestToolResourcesFileSearch` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyAssistantRequestToolResourcesFileSearch {
    /// Overrides the [vector store](/docs/api-reference/vector-stores/object) attached to this assistant.
    /// There can be a maximum of 1 vector store attached to the assistant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_store_ids: Option<Vec<String>>,
}
