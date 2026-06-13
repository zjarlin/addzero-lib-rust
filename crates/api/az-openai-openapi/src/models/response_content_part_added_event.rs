// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseContentPartAddedEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OutputContent,
};

/// Emitted when a new content part is added.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseContentPartAddedEvent {
    /// The type of the event. Always `response.content_part.added`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the output item that the content part was added to.
    pub item_id: String,
    /// The index of the output item that the content part was added to.
    pub output_index: i32,
    /// The index of the content part that was added.
    pub content_index: i32,
    /// The content part that was added.
    pub part: OutputContent,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
