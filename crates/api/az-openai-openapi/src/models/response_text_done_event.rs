// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseTextDoneEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseLogProb,
};

/// Emitted when text content is finalized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTextDoneEvent {
    /// The type of the event. Always `response.output_text.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the output item that the text content is finalized.
    pub item_id: String,
    /// The index of the output item that the text content is finalized.
    pub output_index: i32,
    /// The index of the content part that the text content is finalized.
    pub content_index: i32,
    /// The text content that is finalized.
    pub text: String,
    /// The sequence number for this event.
    pub sequence_number: i32,
    /// The log probabilities of the tokens in the delta.
    pub logprobs: Vec<ResponseLogProb>,
}
