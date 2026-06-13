// Generated from OpenAPI spec. Do not edit by hand.
//! `ApplyPatchToolParam` DTO.

use serde::{Deserialize, Serialize};

/// Allows the assistant to create, delete, or update files using unified diffs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyPatchToolParam {
    /// The type of the tool. Always `apply_patch`.
    #[serde(rename = "type")]
    pub type_value: String,
}
