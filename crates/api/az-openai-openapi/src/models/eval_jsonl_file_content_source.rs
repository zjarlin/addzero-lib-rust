// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalJsonlFileContentSource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalJsonlFileContentSourceContentItem,
};

/// EvalJsonlFileContentSource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalJsonlFileContentSource {
    /// The type of jsonl source. Always `file_content`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The content of the jsonl file.
    pub content: Vec<EvalJsonlFileContentSourceContentItem>,
}
