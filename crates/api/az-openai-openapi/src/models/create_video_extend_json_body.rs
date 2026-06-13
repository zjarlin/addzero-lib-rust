// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateVideoExtendJsonBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VideoReferenceInputParam,
    VideoSeconds,
};

/// JSON parameters for extending an existing generated video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoExtendJsonBody {
    /// Reference to the completed video to extend.
    pub video: VideoReferenceInputParam,
    /// Updated text prompt that directs the extension generation.
    pub prompt: String,
    /// Length of the newly generated extension segment in seconds (allowed values: 4, 8, 12, 16, 20).
    pub seconds: VideoSeconds,
}
