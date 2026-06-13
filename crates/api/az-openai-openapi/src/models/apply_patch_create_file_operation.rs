// Generated from OpenAPI spec. Do not edit by hand.
//! `ApplyPatchCreateFileOperation` DTO.

use serde::{Deserialize, Serialize};

/// Instruction describing how to create a file via the apply_patch tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyPatchCreateFileOperation {
    /// Create a new file with the provided diff.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Path of the file to create.
    pub path: String,
    /// Diff to apply.
    pub diff: String,
}
