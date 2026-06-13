// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseTextDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseLogProb,
};

/// Emitted when there is an additional text delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTextDeltaEvent {
    /// The type of the event. Always `response.output_text.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the output item that the text delta was added to.
    pub item_id: String,
    /// The index of the output item that the text delta was added to.
    pub output_index: i32,
    /// The index of the content part that the text delta was added to.
    pub content_index: i32,
    /// The text delta that was added.
    pub delta: String,
    /// The sequence number for this event.
    pub sequence_number: i32,
    /// The log probabilities of the tokens in the delta.
    pub logprobs: Vec<ResponseLogProb>,
}
