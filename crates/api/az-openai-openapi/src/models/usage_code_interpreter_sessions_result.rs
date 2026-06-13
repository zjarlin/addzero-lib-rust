// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UsageCodeInterpreterSessionsResult` DTO.

use serde::{Deserialize, Serialize};

/// The aggregated code interpreter sessions usage details of the specific time bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCodeInterpreterSessionsResult {
    pub object: String,
    /// The number of code interpreter sessions.
    pub num_sessions: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}
