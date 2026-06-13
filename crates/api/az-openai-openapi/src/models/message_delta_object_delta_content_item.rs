// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageDeltaObjectDeltaContentItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageDeltaContentImageFileObject,
    MessageDeltaContentImageUrlObject,
    MessageDeltaContentRefusalObject,
    MessageDeltaContentTextObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageDeltaObjectDeltaContentItem {
    MessageDeltaContentImageFileObject(MessageDeltaContentImageFileObject),
    MessageDeltaContentTextObject(MessageDeltaContentTextObject),
    MessageDeltaContentRefusalObject(MessageDeltaContentRefusalObject),
    MessageDeltaContentImageUrlObject(MessageDeltaContentImageUrlObject),
}
