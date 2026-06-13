// Generated from OpenAPI spec. Do not edit by hand.
//! `AssistantToolsFileSearch` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantToolsFileSearchFileSearch,
};

/// FileSearch tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantToolsFileSearch {
    /// The type of tool being defined: `file_search`
    #[serde(rename = "type")]
    pub type_value: String,
    /// Overrides for the file search tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: Option<AssistantToolsFileSearchFileSearch>,
}
