// Generated from OpenAPI spec. Do not edit by hand.
//! `DoubleClickAction` DTO.

use serde::{Deserialize, Serialize};

/// A double click action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubleClickAction {
    /// Specifies the event type. For a double click action, this property is always set to `double_click`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The x-coordinate where the double click occurred.
    pub x: i32,
    /// The y-coordinate where the double click occurred.
    pub y: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
}
