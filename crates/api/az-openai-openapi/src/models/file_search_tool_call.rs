// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FileSearchToolCall` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FileSearchToolCallResult,
};

/// The results of a file search tool call. See the [file search guide](/docs/guides/tools-file-search)
/// for more information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchToolCall {
    /// The unique ID of the file search tool call.
    pub id: String,
    /// The type of the file search tool call. Always `file_search_call`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The status of the file search tool call. One of `in_progress`, `searching`, `incomplete` or
    /// `failed`,
    pub status: String,
    /// The queries used to search for files.
    pub queries: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<FileSearchToolCallResult>>,
}
