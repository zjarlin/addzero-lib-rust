// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateVideoMultipartBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateVideoMultipartBodyInputReference,
    VideoModel,
    VideoSeconds,
    VideoSize,
};

/// Multipart parameters for creating a new video generation job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoMultipartBody {
    /// The video generation model to use (allowed values: sora-2, sora-2-pro). Defaults to `sora-2`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<VideoModel>,
    /// Text prompt that describes the video to generate.
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_reference: Option<CreateVideoMultipartBodyInputReference>,
    /// Clip duration in seconds (allowed values: 4, 8, 12). Defaults to 4 seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<VideoSeconds>,
    /// Output resolution formatted as width x height (allowed values: 720x1280, 1280x720, 1024x1792,
    /// 1792x1024). Defaults to 720x1280.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<VideoSize>,
}
