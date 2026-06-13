// Generated from OpenAPI spec. Do not edit by hand.
//! `ThreadObjectToolResources` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ThreadObjectToolResourcesCodeInterpreter,
    ThreadObjectToolResourcesFileSearch,
};

/// A set of resources that are made available to the assistant's tools in this thread. The resources
/// are specific to the type of tool. For example, the `code_interpreter` tool requires a list of file
/// IDs, while the `file_search` tool requires a list of vector store IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadObjectToolResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: Option<ThreadObjectToolResourcesCodeInterpreter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: Option<ThreadObjectToolResourcesFileSearch>,
}
