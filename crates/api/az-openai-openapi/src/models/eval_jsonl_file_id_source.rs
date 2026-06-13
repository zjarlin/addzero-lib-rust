// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalJsonlFileIdSource` DTO.

use serde::{Deserialize, Serialize};

/// EvalJsonlFileIdSource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalJsonlFileIdSource {
    /// The type of jsonl source. Always `file_id`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The identifier of the file.
    pub id: String,
}
