// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateVideoEditJsonBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VideoReferenceInputParam,
};

/// JSON parameters for editing an existing generated video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoEditJsonBody {
    /// Reference to the completed video to edit.
    pub video: VideoReferenceInputParam,
    /// Text prompt that describes how to edit the source video.
    pub prompt: String,
}
