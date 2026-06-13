// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ApplyPatchUpdateFileOperation` DTO.

use serde::{Deserialize, Serialize};

/// Instruction describing how to update a file via the apply_patch tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyPatchUpdateFileOperation {
    /// Update an existing file with the provided diff.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Path of the file to update.
    pub path: String,
    /// Diff to apply.
    pub diff: String,
}
