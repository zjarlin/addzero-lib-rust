// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateImageEditRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

use crate::models::{
    CreateImageEditRequestImage,
    InputFidelity,
    PartialImages,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateImageEditRequest {
    /// The image(s) to edit. Must be a supported image file or an array of images. For the GPT image models
    /// (`gpt-image-1`, `gpt-image-1-mini`, and `gpt-image-1.5`), each image should be a `png`, `webp`, or
    /// `jpg` file less than 50MB. You can provide up to 16 images. `chatgpt-image-latest` follows the same
    /// input constraints as GPT image models. For `dall-e-2`, you can only provide one image, and it should
    /// be a square `png` file less than 4MB.
    pub image: CreateImageEditRequestImage,
    /// A text description of the desired image(s). The maximum length is 1000 characters for `dall-e-2`,
    /// and 32000 characters for the GPT image models.
    pub prompt: String,
    /// An additional image whose fully transparent areas (e.g. where alpha is zero) indicate where `image`
    /// should be edited. If there are multiple images provided, the mask will be applied on the first
    /// image. Must be a valid PNG file, less than 4MB, and have the same dimensions as `image`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: Option<OpenAiBinaryBody>,
    /// Allows to set transparency for the background of the generated image(s). This parameter is only
    /// supported for the GPT image models. Must be one of `transparent`, `opaque` or `auto` (default
    /// value). When `auto` is used, the model will automatically determine the best background for the
    /// image. If `transparent`, the output format needs to support transparency, so it should be set to
    /// either `png` (default value) or `webp`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// The model to use for image generation. Defaults to `gpt-image-1.5`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The number of images to generate. Must be between 1 and 10.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,
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
    /// The format in which the generated images are returned. Must be one of `url` or `b64_json`. URLs are
    /// only valid for 60 minutes after the image has been generated. This parameter is only supported for
    /// `dall-e-2` (default is `url` for `dall-e-2`), as GPT image models always return base64-encoded
    /// images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// The format in which the generated images are returned. This parameter is only supported for the GPT
    /// image models. Must be one of `png`, `jpeg`, or `webp`. The default value is `png`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// The compression level (0-100%) for the generated images. This parameter is only supported for the
    /// GPT image models with the `webp` or `jpeg` output formats, and defaults to 100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<i32>,
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    /// [Learn more](/docs/guides/safety-best-practices#end-user-ids).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_fidelity: Option<InputFidelity>,
    /// Edit the image in streaming mode. Defaults to `false`. See the [Image generation
    /// guide](/docs/guides/image-generation) for more information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_images: Option<PartialImages>,
    /// The quality of the image that will be generated for GPT image models. Defaults to `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
}
