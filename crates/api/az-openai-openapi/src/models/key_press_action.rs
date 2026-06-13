// Generated from OpenAPI spec. Do not edit by hand.
//! `KeyPressAction` DTO.

use serde::{Deserialize, Serialize};

/// A collection of keypresses the model would like to perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPressAction {
    /// Specifies the event type. For a keypress action, this property is always set to `keypress`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The combination of keys the model is requesting to be pressed. This is an array of strings, each
    /// representing a key.
    pub keys: Vec<String>,
}
