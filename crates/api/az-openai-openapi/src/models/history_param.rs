// Generated from OpenAPI spec. Do not edit by hand.
//! `HistoryParam` DTO.

use serde::{Deserialize, Serialize};

/// Controls how much historical context is retained for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryParam {
    /// Enables chat users to access previous ChatKit threads. Defaults to true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Number of recent ChatKit threads users have access to. Defaults to unlimited when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recent_threads: Option<i32>,
}
