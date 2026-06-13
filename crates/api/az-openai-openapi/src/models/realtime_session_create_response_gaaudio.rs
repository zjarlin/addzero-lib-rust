// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeSessionCreateResponseGAAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateResponseGAAudioInput,
    RealtimeSessionCreateResponseGAAudioOutput,
};

/// Configuration for input and output audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateResponseGAAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<RealtimeSessionCreateResponseGAAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<RealtimeSessionCreateResponseGAAudioOutput>,
}
