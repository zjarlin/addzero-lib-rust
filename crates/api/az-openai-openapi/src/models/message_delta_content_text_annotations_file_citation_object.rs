// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageDeltaContentTextAnnotationsFileCitationObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageDeltaContentTextAnnotationsFileCitationObjectFileCitation,
};

/// A citation within the message that points to a specific quote from a specific File associated with
/// the assistant or the message. Generated when the assistant uses the "file_search" tool to search
/// files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaContentTextAnnotationsFileCitationObject {
    /// The index of the annotation in the text content part.
    pub index: i32,
    /// Always `file_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text in the message content that needs to be replaced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_citation: Option<MessageDeltaContentTextAnnotationsFileCitationObjectFileCitation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
}
