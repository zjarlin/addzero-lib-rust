// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ImageGenInputUsageDetails` DTO.

use serde::{Deserialize, Serialize};

/// The input tokens detailed information for the image generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenInputUsageDetails {
    /// The number of text tokens in the input prompt.
    pub text_tokens: i32,
    /// The number of image tokens in the input prompt.
    pub image_tokens: i32,
}
