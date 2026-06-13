// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatSessionHistory` DTO.

use serde::{Deserialize, Serialize};

/// History retention preferences returned for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionHistory {
    /// Indicates if chat history is persisted for the session.
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recent_threads: Option<i32>,
}
