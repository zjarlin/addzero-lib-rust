// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDetailsToolCallsFileSearchResultObjectContentItem` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsToolCallsFileSearchResultObjectContentItem {
    /// The type of the content.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The text content of the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}
