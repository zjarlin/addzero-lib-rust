// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ModifyAssistantRequestToolResources` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ModifyAssistantRequestToolResourcesCodeInterpreter,
    ModifyAssistantRequestToolResourcesFileSearch,
};

/// A set of resources that are used by the assistant's tools. The resources are specific to the type of
/// tool. For example, the `code_interpreter` tool requires a list of file IDs, while the `file_search`
/// tool requires a list of vector store IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyAssistantRequestToolResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: Option<ModifyAssistantRequestToolResourcesCodeInterpreter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: Option<ModifyAssistantRequestToolResourcesFileSearch>,
}
