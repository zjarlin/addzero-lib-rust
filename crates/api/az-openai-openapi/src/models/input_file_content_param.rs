// Generated from OpenAPI spec. Do not edit by hand.
//! `InputFileContentParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FileDetailEnum,
};

/// A file input to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFileContentParam {
    /// The type of the input item. Always `input_file`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_url: Option<String>,
    /// The detail level of the file to be sent to the model. Use `low` for the default rendering behavior,
    /// or `high` to render the file at higher quality. Defaults to `low`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<FileDetailEnum>,
}
