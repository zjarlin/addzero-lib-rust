// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeReasoning` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeReasoningEffort,
};

/// Configuration for reasoning-capable Realtime models such as `gpt-realtime-2`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeReasoning {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<RealtimeReasoningEffort>,
}
