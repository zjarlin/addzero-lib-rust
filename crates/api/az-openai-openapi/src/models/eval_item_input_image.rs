// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalItemInputImage` DTO.

use serde::{Deserialize, Serialize};

/// An image input block used within EvalItem content arrays.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalItemInputImage {
    /// The type of the image input. Always `input_image`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The URL of the image input.
    pub image_url: String,
    /// The detail level of the image to be sent to the model. One of `high`, `low`, or `auto`. Defaults to
    /// `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
