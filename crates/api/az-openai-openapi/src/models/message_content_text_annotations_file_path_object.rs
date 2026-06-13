// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageContentTextAnnotationsFilePathObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageContentTextAnnotationsFilePathObjectFilePath,
};

/// A URL for the file that's generated when the assistant used the `code_interpreter` tool to generate
/// a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentTextAnnotationsFilePathObject {
    /// Always `file_path`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text in the message content that needs to be replaced.
    pub text: String,
    pub file_path: MessageContentTextAnnotationsFilePathObjectFilePath,
    pub start_index: i32,
    pub end_index: i32,
}
