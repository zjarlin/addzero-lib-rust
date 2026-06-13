// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateImageVariationRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateImageVariationRequest {
    /// The image to use as the basis for the variation(s). Must be a valid PNG file, less than 4MB, and
    /// square.
    pub image: OpenAiBinaryBody,
    /// The model to use for image generation. Only `dall-e-2` is supported at this time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The number of images to generate. Must be between 1 and 10.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,
    /// The format in which the generated images are returned. Must be one of `url` or `b64_json`. URLs are
    /// only valid for 60 minutes after the image has been generated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// The size of the generated images. Must be one of `256x256`, `512x512`, or `1024x1024`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    /// [Learn more](/docs/guides/safety-best-practices#end-user-ids).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}
