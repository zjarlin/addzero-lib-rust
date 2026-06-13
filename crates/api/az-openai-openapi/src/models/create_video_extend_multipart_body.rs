// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateVideoExtendMultipartBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateVideoExtendMultipartBodyVideo,
    VideoSeconds,
};

/// Multipart parameters for extending an existing generated video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoExtendMultipartBody {
    pub video: CreateVideoExtendMultipartBodyVideo,
    /// Updated text prompt that directs the extension generation.
    pub prompt: String,
    /// Length of the newly generated extension segment in seconds (allowed values: 4, 8, 12, 16, 20).
    pub seconds: VideoSeconds,
}
