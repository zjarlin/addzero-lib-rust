// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateImageEditRequestImage` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

/// The image(s) to edit. Must be a supported image file or an array of images. For the GPT image models
/// (`gpt-image-1`, `gpt-image-1-mini`, and `gpt-image-1.5`), each image should be a `png`, `webp`, or
/// `jpg` file less than 50MB. You can provide up to 16 images. `chatgpt-image-latest` follows the same
/// input constraints as GPT image models. For `dall-e-2`, you can only provide one image, and it should
/// be a square `png` file less than 4MB.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateImageEditRequestImage {
    String(OpenAiBinaryBody),
    Array(Vec<OpenAiBinaryBody>),
}
