// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseOutputItemDoneEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OutputItem,
};

/// Emitted when an output item is marked done.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseOutputItemDoneEvent {
    /// The type of the event. Always `response.output_item.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item that was marked done.
    pub output_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The output item that was marked done.
    pub item: OutputItem,
}
