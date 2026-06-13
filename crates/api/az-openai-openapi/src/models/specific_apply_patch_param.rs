// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `SpecificApplyPatchParam` DTO.

use serde::{Deserialize, Serialize};

/// Forces the model to call the apply_patch tool when executing a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificApplyPatchParam {
    /// The tool to call. Always `apply_patch`.
    #[serde(rename = "type")]
    pub type_value: String,
}
