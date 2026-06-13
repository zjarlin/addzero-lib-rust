// Generated from OpenAPI spec. Do not edit by hand.
//! `ApplyPatchUpdateFileOperationParam` DTO.

use serde::{Deserialize, Serialize};

/// Instruction for updating an existing file via the apply_patch tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyPatchUpdateFileOperationParam {
    /// The operation type. Always `update_file`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Path of the file to update relative to the workspace root.
    pub path: String,
    /// Unified diff content to apply to the existing file.
    pub diff: String,
}
