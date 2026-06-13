// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateAssistantRequestToolResourcesCodeInterpreter` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssistantRequestToolResourcesCodeInterpreter {
    /// A list of [file](/docs/api-reference/files) IDs made available to the `code_interpreter` tool. There
    /// can be a maximum of 20 files associated with the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
}
