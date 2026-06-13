// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebhookBatchFailedData` DTO.

use serde::{Deserialize, Serialize};

/// Event data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookBatchFailedData {
    /// The unique ID of the batch API request.
    pub id: String,
}
