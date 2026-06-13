// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `InputFileContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FileInputDetail,
};

/// A file input to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFileContent {
    /// The type of the input item. Always `input_file`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// The name of the file to be sent to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// The content of the file to be sent to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    /// The URL of the file to be sent to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_url: Option<String>,
    /// The detail level of the file to be sent to the model. Use `low` for the default rendering behavior,
    /// or `high` to render the file at higher quality. Defaults to `low`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<FileInputDetail>,
}
