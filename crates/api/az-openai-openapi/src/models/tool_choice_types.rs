// Generated from OpenAPI spec. Do not edit by hand.
//! `ToolChoiceTypes` DTO.

use serde::{Deserialize, Serialize};

/// Indicates that the model should use a built-in tool to generate a response. [Learn more about built-
/// in tools](/docs/guides/tools).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceTypes {
    /// The type of hosted tool the model should to use. Learn more about [built-in
    /// tools](/docs/guides/tools). Allowed values are: - `file_search` - `web_search_preview` - `computer`
    /// - `computer_use_preview` - `computer_use` - `code_interpreter` - `image_generation`
    #[serde(rename = "type")]
    pub type_value: String,
}
