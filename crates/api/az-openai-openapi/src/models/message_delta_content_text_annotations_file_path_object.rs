// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageDeltaContentTextAnnotationsFilePathObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageDeltaContentTextAnnotationsFilePathObjectFilePath,
};

/// A URL for the file that's generated when the assistant used the `code_interpreter` tool to generate
/// a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaContentTextAnnotationsFilePathObject {
    /// The index of the annotation in the text content part.
    pub index: i32,
    /// Always `file_path`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text in the message content that needs to be replaced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<MessageDeltaContentTextAnnotationsFilePathObjectFilePath>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
}
