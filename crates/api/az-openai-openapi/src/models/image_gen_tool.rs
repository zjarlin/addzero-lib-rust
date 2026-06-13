// Generated from OpenAPI spec. Do not edit by hand.
//! `ImageGenTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ImageGenActionEnum,
    ImageGenToolInputImageMask,
    InputFidelity,
};

/// A tool that generates images using the GPT image models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenTool {
    /// The type of the image generation tool. Always `image_generation`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The quality of the generated image. One of `low`, `medium`, `high`, or `auto`. Default: `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// The size of the generated images. For `gpt-image-2` and `gpt-image-2-2026-04-21`, arbitrary
    /// resolutions are supported as `WIDTHxHEIGHT` strings, for example `1536x864`. Width and height must
    /// both be divisible by 16 and the requested aspect ratio must be between 1:3 and 3:1. Resolutions
    /// above `2560x1440` are experimental, and the maximum supported resolution is `3840x2160`. The
    /// requested size must also satisfy the model's current pixel and edge limits. The standard sizes
    /// `1024x1024`, `1536x1024`, and `1024x1536` are supported by the GPT image models; `auto` is supported
    /// for models that allow automatic sizing. For `dall-e-2`, use one of `256x256`, `512x512`, or
    /// `1024x1024`. For `dall-e-3`, use one of `1024x1024`, `1792x1024`, or `1024x1792`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// The output format of the generated image. One of `png`, `webp`, or `jpeg`. Default: `png`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// Compression level for the output image. Default: 100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<i32>,
    /// Moderation level for the generated image. Default: `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moderation: Option<String>,
    /// Background type for the generated image. One of `transparent`, `opaque`, or `auto`. Default: `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_fidelity: Option<InputFidelity>,
    /// Optional mask for inpainting. Contains `image_url` (string, optional) and `file_id` (string,
    /// optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_image_mask: Option<ImageGenToolInputImageMask>,
    /// Number of partial images to generate in streaming mode, from 0 (default value) to 3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_images: Option<i32>,
    /// Whether to generate a new image or edit an existing image. Default: `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<ImageGenActionEnum>,
}
