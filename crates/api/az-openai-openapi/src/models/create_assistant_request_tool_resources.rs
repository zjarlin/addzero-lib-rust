// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateAssistantRequestToolResources` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateAssistantRequestToolResourcesCodeInterpreter,
    CreateAssistantRequestToolResourcesFileSearch,
};

/// A set of resources that are used by the assistant's tools. The resources are specific to the type of
/// tool. For example, the `code_interpreter` tool requires a list of file IDs, while the `file_search`
/// tool requires a list of vector store IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssistantRequestToolResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: Option<CreateAssistantRequestToolResourcesCodeInterpreter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: Option<CreateAssistantRequestToolResourcesFileSearch>,
}
