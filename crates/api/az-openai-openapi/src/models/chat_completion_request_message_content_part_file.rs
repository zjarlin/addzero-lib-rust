// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestMessageContentPartFile` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestMessageContentPartFileFile,
};

/// Learn about [file inputs](/docs/guides/text) for text generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestMessageContentPartFile {
    /// The type of the content part. Always `file`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub file: ChatCompletionRequestMessageContentPartFileFile,
}
