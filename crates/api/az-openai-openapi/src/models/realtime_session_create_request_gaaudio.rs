// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeSessionCreateRequestGAAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateRequestGAAudioInput,
    RealtimeSessionCreateRequestGAAudioOutput,
};

/// Configuration for input and output audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateRequestGAAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<RealtimeSessionCreateRequestGAAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<RealtimeSessionCreateRequestGAAudioOutput>,
}
