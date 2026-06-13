// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FilePath` DTO.

use serde::{Deserialize, Serialize};

/// A path to a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePath {
    /// The type of the file path. Always `file_path`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the file.
    pub file_id: String,
    /// The index of the file in the list of files.
    pub index: i32,
}
