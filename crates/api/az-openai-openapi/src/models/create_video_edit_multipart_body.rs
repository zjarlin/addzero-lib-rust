// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateVideoEditMultipartBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateVideoEditMultipartBodyVideo,
};

/// Parameters for editing an existing generated video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoEditMultipartBody {
    pub video: CreateVideoEditMultipartBodyVideo,
    /// Text prompt that describes how to edit the source video.
    pub prompt: String,
}
