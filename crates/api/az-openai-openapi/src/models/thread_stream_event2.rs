// Generated from OpenAPI spec. Do not edit by hand.
//! `ThreadStreamEvent2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ThreadObject,
};

/// Occurs when a new [thread](/docs/api-reference/threads/object) is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadStreamEvent2 {
    /// Whether to enable input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    pub event: String,
    pub data: ThreadObject,
}
