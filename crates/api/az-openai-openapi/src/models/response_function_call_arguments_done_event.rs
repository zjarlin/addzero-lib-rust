// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseFunctionCallArgumentsDoneEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when function-call arguments are finalized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFunctionCallArgumentsDoneEvent {
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item.
    pub item_id: String,
    /// The name of the function that was called.
    pub name: String,
    /// The index of the output item.
    pub output_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The function-call arguments.
    pub arguments: String,
}
