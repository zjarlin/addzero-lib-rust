// Generated from OpenAPI spec. Do not edit by hand.
//! `ImagesResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Image,
    ImageGenUsage,
};

/// The response from the image generation endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagesResponse {
    /// The Unix timestamp (in seconds) of when the image was created.
    pub created: i64,
    /// The list of generated images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<Image>>,
    /// The background parameter used for the image generation. Either `transparent` or `opaque`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// The output format of the image generation. Either `png`, `webp`, or `jpeg`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// The size of the image generated. Either `1024x1024`, `1024x1536`, or `1536x1024`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// The quality of the image generated. Either `low`, `medium`, or `high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<ImageGenUsage>,
}
