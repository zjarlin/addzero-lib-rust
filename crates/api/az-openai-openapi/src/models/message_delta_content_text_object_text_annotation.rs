// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageDeltaContentTextObjectTextAnnotation` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageDeltaContentTextAnnotationsFileCitationObject,
    MessageDeltaContentTextAnnotationsFilePathObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageDeltaContentTextObjectTextAnnotation {
    MessageDeltaContentTextAnnotationsFileCitationObject(MessageDeltaContentTextAnnotationsFileCitationObject),
    MessageDeltaContentTextAnnotationsFilePathObject(MessageDeltaContentTextAnnotationsFilePathObject),
}
