// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestMessageContentPartImage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestMessageContentPartImageImageUrl,
};

/// Learn about [image inputs](/docs/guides/vision).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestMessageContentPartImage {
    /// The type of the content part.
    #[serde(rename = "type")]
    pub type_value: String,
    pub image_url: ChatCompletionRequestMessageContentPartImageImageUrl,
}
