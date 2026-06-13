// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateMessageRequestContentArrayOfContentPart` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageContentImageFileObject,
    MessageContentImageUrlObject,
    MessageRequestContentTextObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateMessageRequestContentArrayOfContentPart {
    MessageContentImageFileObject(MessageContentImageFileObject),
    MessageContentImageUrlObject(MessageContentImageUrlObject),
    MessageRequestContentTextObject(MessageRequestContentTextObject),
}
