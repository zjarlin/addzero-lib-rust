// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FileCitationBody` DTO.

use serde::{Deserialize, Serialize};

/// A citation to a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCitationBody {
    /// The type of the file citation. Always `file_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the file.
    pub file_id: String,
    /// The index of the file in the list of files.
    pub index: i32,
    /// The filename of the file cited.
    pub filename: String,
}
