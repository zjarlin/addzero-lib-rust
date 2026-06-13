// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UsageImagesResult` DTO.

use serde::{Deserialize, Serialize};

/// The aggregated images usage details of the specific time bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageImagesResult {
    pub object: String,
    /// The number of images processed.
    pub images: i32,
    /// The count of requests made to the model.
    pub num_model_requests: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}
