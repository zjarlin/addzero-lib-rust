// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionList` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateChatCompletionResponse,
};

/// An object representing a list of Chat Completions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionList {
    /// The type of this object. It is always set to "list".
    pub object: String,
    /// An array of chat completion objects.
    pub data: Vec<CreateChatCompletionResponse>,
    /// The identifier of the first chat completion in the data array.
    pub first_id: String,
    /// The identifier of the last chat completion in the data array.
    pub last_id: String,
    /// Indicates whether there are more Chat Completions available.
    pub has_more: bool,
}
