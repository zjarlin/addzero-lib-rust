// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateMessageRequestContentItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageContentImageFileObject,
    MessageContentImageUrlObject,
    MessageRequestContentTextObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateMessageRequestContentItem {
    MessageContentImageFileObject(MessageContentImageFileObject),
    MessageContentImageUrlObject(MessageContentImageUrlObject),
    MessageRequestContentTextObject(MessageRequestContentTextObject),
}
