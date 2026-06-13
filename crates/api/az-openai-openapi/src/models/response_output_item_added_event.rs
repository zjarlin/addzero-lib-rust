// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseOutputItemAddedEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OutputItem,
};

/// Emitted when a new output item is added.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseOutputItemAddedEvent {
    /// The type of the event. Always `response.output_item.added`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item that was added.
    pub output_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The output item that was added.
    pub item: OutputItem,
}
