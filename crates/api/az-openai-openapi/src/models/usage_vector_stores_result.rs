// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UsageVectorStoresResult` DTO.

use serde::{Deserialize, Serialize};

/// The aggregated vector stores usage details of the specific time bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageVectorStoresResult {
    pub object: String,
    /// The vector stores usage in bytes.
    pub usage_bytes: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}
