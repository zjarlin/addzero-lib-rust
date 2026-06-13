// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageDeltaContentTextObjectText` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageDeltaContentTextObjectTextAnnotation,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaContentTextObjectText {
    /// The data that makes up the text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<MessageDeltaContentTextObjectTextAnnotation>>,
}
