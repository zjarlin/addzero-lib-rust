// Generated from OpenAPI spec. Do not edit by hand.
//! `TypeParam` DTO.

use serde::{Deserialize, Serialize};

/// An action to type in text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeParam {
    /// Specifies the event type. For a type action, this property is always set to `type`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text to type.
    pub text: String,
}
