// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageObjectContentItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageContentImageFileObject,
    MessageContentImageUrlObject,
    MessageContentRefusalObject,
    MessageContentTextObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageObjectContentItem {
    MessageContentImageFileObject(MessageContentImageFileObject),
    MessageContentImageUrlObject(MessageContentImageUrlObject),
    MessageContentTextObject(MessageContentTextObject),
    MessageContentRefusalObject(MessageContentRefusalObject),
}
