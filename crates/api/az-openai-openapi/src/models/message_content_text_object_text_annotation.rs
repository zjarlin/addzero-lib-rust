// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageContentTextObjectTextAnnotation` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageContentTextAnnotationsFileCitationObject,
    MessageContentTextAnnotationsFilePathObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContentTextObjectTextAnnotation {
    MessageContentTextAnnotationsFileCitationObject(MessageContentTextAnnotationsFileCitationObject),
    MessageContentTextAnnotationsFilePathObject(MessageContentTextAnnotationsFilePathObject),
}
