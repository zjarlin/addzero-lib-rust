// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ApplyPatchDeleteFileOperationParam` DTO.

use serde::{Deserialize, Serialize};

/// Instruction for deleting an existing file via the apply_patch tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyPatchDeleteFileOperationParam {
    /// The operation type. Always `delete_file`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Path of the file to delete relative to the workspace root.
    pub path: String,
}
