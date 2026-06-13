// Generated from OpenAPI spec. Do not edit by hand.
//! `ImagesUsageInputTokensDetails` DTO.

use serde::{Deserialize, Serialize};

/// The input tokens detailed information for the image generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagesUsageInputTokensDetails {
    /// The number of text tokens in the input prompt.
    pub text_tokens: i32,
    /// The number of image tokens in the input prompt.
    pub image_tokens: i32,
}
