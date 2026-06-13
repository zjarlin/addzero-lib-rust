// Generated from OpenAPI spec. Do not edit by hand.
//! `ClickParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ClickButtonType,
};

/// A click action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickParam {
    /// Specifies the event type. For a click action, this property is always `click`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Indicates which mouse button was pressed during the click. One of `left`, `right`, `wheel`, `back`,
    /// or `forward`.
    pub button: ClickButtonType,
    /// The x-coordinate where the click occurred.
    pub x: i32,
    /// The y-coordinate where the click occurred.
    pub y: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
}
