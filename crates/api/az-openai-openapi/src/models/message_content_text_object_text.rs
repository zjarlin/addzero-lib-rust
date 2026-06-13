// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageContentTextObjectText` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageContentTextObjectTextAnnotation,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentTextObjectText {
    /// The data that makes up the text.
    pub value: String,
    pub annotations: Vec<MessageContentTextObjectTextAnnotation>,
}
