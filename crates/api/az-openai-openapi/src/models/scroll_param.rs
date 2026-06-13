// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ScrollParam` DTO.

use serde::{Deserialize, Serialize};

/// A scroll action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollParam {
    /// Specifies the event type. For a scroll action, this property is always set to `scroll`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The x-coordinate where the scroll occurred.
    pub x: i32,
    /// The y-coordinate where the scroll occurred.
    pub y: i32,
    /// The horizontal scroll distance.
    pub scroll_x: i32,
    /// The vertical scroll distance.
    pub scroll_y: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
}
