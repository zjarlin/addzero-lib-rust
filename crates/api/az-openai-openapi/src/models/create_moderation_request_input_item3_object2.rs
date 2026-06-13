// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateModerationRequestInputItem3Object2` DTO.

use serde::{Deserialize, Serialize};

/// An object describing text to classify.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationRequestInputItem3Object2 {
    /// Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A string of text to classify.
    pub text: String,
}
