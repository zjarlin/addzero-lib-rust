// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ComputerScreenshotImage` DTO.

use serde::{Deserialize, Serialize};

/// A computer screenshot image used with the computer use tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerScreenshotImage {
    /// Specifies the event type. For a computer screenshot, this property is always set to
    /// `computer_screenshot`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The URL of the screenshot image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// The identifier of an uploaded file that contains the screenshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}
