// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ApplyPatchCreateFileOperationParam` DTO.

use serde::{Deserialize, Serialize};

/// Instruction for creating a new file via the apply_patch tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyPatchCreateFileOperationParam {
    /// The operation type. Always `create_file`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Path of the file to create relative to the workspace root.
    pub path: String,
    /// Unified diff content to apply when creating the file.
    pub diff: String,
}
