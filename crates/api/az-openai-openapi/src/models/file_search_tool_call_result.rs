// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FileSearchToolCallResult` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VectorStoreFileAttributes,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchToolCallResult {
    /// The unique ID of the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// The text that was retrieved from the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// The name of the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<VectorStoreFileAttributes>,
    /// The relevance score of the file - a value between 0 and 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}
