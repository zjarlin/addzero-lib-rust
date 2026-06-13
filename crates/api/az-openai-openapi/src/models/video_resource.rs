// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VideoResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Error2,
    VideoModel,
    VideoSize,
    VideoStatus,
};

/// Structured information describing a generated video job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoResource {
    /// Unique identifier for the video job.
    pub id: String,
    /// The object type, which is always `video`.
    pub object: String,
    /// The video generation model that produced the job.
    pub model: VideoModel,
    /// Current lifecycle status of the video job.
    pub status: VideoStatus,
    /// Approximate completion percentage for the generation task.
    pub progress: i32,
    /// Unix timestamp (seconds) for when the job was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// The resolution of the generated video.
    pub size: VideoSize,
    /// Duration of the generated clip in seconds. For extensions, this is the stitched total duration.
    pub seconds: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remixed_from_video_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Error2>,
}
