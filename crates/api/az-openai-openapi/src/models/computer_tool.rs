// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ComputerTool` DTO.

use serde::{Deserialize, Serialize};

/// A tool that controls a virtual computer. Learn more about the [computer
/// tool](https://platform.openai.com/docs/guides/tools-computer-use).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerTool {
    /// The type of the computer tool. Always `computer`.
    #[serde(rename = "type")]
    pub type_value: String,
}
