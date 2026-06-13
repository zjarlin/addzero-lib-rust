// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageContentTextAnnotationsFileCitationObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageContentTextAnnotationsFileCitationObjectFileCitation,
};

/// A citation within the message that points to a specific quote from a specific File associated with
/// the assistant or the message. Generated when the assistant uses the "file_search" tool to search
/// files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentTextAnnotationsFileCitationObject {
    /// Always `file_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text in the message content that needs to be replaced.
    pub text: String,
    pub file_citation: MessageContentTextAnnotationsFileCitationObjectFileCitation,
    pub start_index: i32,
    pub end_index: i32,
}
