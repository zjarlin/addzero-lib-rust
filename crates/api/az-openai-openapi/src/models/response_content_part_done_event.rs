// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseContentPartDoneEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OutputContent,
};

/// Emitted when a content part is done.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseContentPartDoneEvent {
    /// The type of the event. Always `response.content_part.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the output item that the content part was added to.
    pub item_id: String,
    /// The index of the output item that the content part was added to.
    pub output_index: i32,
    /// The index of the content part that is done.
    pub content_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The content part that is done.
    pub part: OutputContent,
}
