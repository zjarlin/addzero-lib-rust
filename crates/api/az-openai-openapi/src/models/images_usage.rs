// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ImagesUsage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ImagesUsageInputTokensDetails,
};

/// For the GPT image models only, the token usage information for the image generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagesUsage {
    /// The total number of tokens (images and text) used for the image generation.
    pub total_tokens: i32,
    /// The number of tokens (images and text) in the input prompt.
    pub input_tokens: i32,
    /// The number of image tokens in the output image.
    pub output_tokens: i32,
    /// The input tokens detailed information for the image generation.
    pub input_tokens_details: ImagesUsageInputTokensDetails,
}
