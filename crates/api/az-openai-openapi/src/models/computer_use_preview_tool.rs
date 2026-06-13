// Generated from OpenAPI spec. Do not edit by hand.
//! `ComputerUsePreviewTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ComputerEnvironment,
};

/// A tool that controls a virtual computer. Learn more about the [computer
/// tool](https://platform.openai.com/docs/guides/tools-computer-use).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerUsePreviewTool {
    /// The type of the computer use tool. Always `computer_use_preview`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The type of computer environment to control.
    pub environment: ComputerEnvironment,
    /// The width of the computer display.
    pub display_width: i32,
    /// The height of the computer display.
    pub display_height: i32,
}
