// Generated from OpenAPI spec. Do not edit by hand.
//! `ApplyPatchToolCallOperation` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ApplyPatchCreateFileOperation,
    ApplyPatchDeleteFileOperation,
    ApplyPatchUpdateFileOperation,
};

/// One of the create_file, delete_file, or update_file operations applied via apply_patch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApplyPatchToolCallOperation {
    ApplyPatchCreateFileOperation(ApplyPatchCreateFileOperation),
    ApplyPatchDeleteFileOperation(ApplyPatchDeleteFileOperation),
    ApplyPatchUpdateFileOperation(ApplyPatchUpdateFileOperation),
}
