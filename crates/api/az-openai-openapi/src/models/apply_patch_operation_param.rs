// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ApplyPatchOperationParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ApplyPatchCreateFileOperationParam,
    ApplyPatchDeleteFileOperationParam,
    ApplyPatchUpdateFileOperationParam,
};

/// One of the create_file, delete_file, or update_file operations supplied to the apply_patch tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApplyPatchOperationParam {
    ApplyPatchCreateFileOperationParam(ApplyPatchCreateFileOperationParam),
    ApplyPatchDeleteFileOperationParam(ApplyPatchDeleteFileOperationParam),
    ApplyPatchUpdateFileOperationParam(ApplyPatchUpdateFileOperationParam),
}
