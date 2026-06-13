// Generated from OpenAPI spec. Do not edit by hand.
//! `InputParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputItem,
};

/// Text, image, or file inputs to the model, used to generate a response. Learn more: - [Text inputs
/// and outputs](/docs/guides/text) - [Image inputs](/docs/guides/images) - [File
/// inputs](/docs/guides/pdf-files) - [Conversation state](/docs/guides/conversation-state) - [Function
/// calling](/docs/guides/function-calling)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputParam {
    TextInput(String),
    InputItemList(Vec<InputItem>),
}
