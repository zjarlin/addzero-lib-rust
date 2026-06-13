// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestMessageContentPartImageImageUrl` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestMessageContentPartImageImageUrl {
    /// Either a URL of the image or the base64 encoded image data.
    pub url: String,
    /// Specifies the detail level of the image. Learn more in the [Vision guide](/docs/guides/vision#low-
    /// or-high-fidelity-image-understanding).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
