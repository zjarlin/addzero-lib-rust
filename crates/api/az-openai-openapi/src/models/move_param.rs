// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MoveParam` DTO.

use serde::{Deserialize, Serialize};

/// A mouse move action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveParam {
    /// Specifies the event type. For a move action, this property is always set to `move`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The x-coordinate to move to.
    pub x: i32,
    /// The y-coordinate to move to.
    pub y: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
}
