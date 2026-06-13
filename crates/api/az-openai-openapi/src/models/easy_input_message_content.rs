// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EasyInputMessageContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputMessageContentList,
};

/// Text, image, or audio input to the model, used to generate a response. Can also contain previous
/// assistant responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EasyInputMessageContent {
    TextInput(String),
    InputMessageContentList(InputMessageContentList),
}
