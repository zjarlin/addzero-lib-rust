// Generated from OpenAPI spec. Do not edit by hand.
//! `ApplyPatchDeleteFileOperation` DTO.

use serde::{Deserialize, Serialize};

/// Instruction describing how to delete a file via the apply_patch tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyPatchDeleteFileOperation {
    /// Delete the specified file.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Path of the file to delete.
    pub path: String,
}
