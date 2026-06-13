// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ComputerScreenshotContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ImageDetail,
};

/// A screenshot of a computer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerScreenshotContent {
    /// Specifies the event type. For a computer screenshot, this property is always set to
    /// `computer_screenshot`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// The detail level of the screenshot image to be sent to the model. One of `high`, `low`, `auto`, or
    /// `original`. Defaults to `auto`.
    pub detail: ImageDetail,
}
