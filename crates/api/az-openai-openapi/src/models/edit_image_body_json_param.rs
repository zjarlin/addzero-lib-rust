// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EditImageBodyJsonParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ImageRefParam,
    PartialImages,
};

/// JSON request body for image edits. Use `images` (array of `ImageRefParam`) instead of multipart
/// `image` uploads. You can reference images via external URLs, data URLs, or uploaded file IDs. JSON
/// edits support GPT image models only; DALL-E edits require multipart (`dall-e-2` only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditImageBodyJsonParam {
    /// The model to use for image editing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Input image references to edit. For GPT image models, you can provide up to 16 images.
    pub images: Vec<ImageRefParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: Option<ImageRefParam>,
    /// A text description of the desired image edit.
    pub prompt: String,
    /// The number of edited images to generate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,
    /// Output quality for GPT image models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Controls fidelity to the original input image(s).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_fidelity: Option<String>,
    /// Requested output image size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// A unique identifier representing your end-user, which can help OpenAI monitor and detect abuse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Output image format. Supported for GPT image models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// Compression level for `jpeg` or `webp` output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<i32>,
    /// Moderation level for GPT image models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moderation: Option<String>,
    /// Background behavior for generated image output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// Stream partial image results as events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_images: Option<PartialImages>,
}
