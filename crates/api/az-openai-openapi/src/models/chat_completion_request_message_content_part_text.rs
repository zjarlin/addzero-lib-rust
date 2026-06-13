// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionRequestMessageContentPartText` DTO.

use serde::{Deserialize, Serialize};

/// Learn about [text inputs](/docs/guides/text-generation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestMessageContentPartText {
    /// The type of the content part.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text content.
    pub text: String,
}
