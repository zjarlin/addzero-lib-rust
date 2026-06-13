// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebhookEvalRunSucceededData` DTO.

use serde::{Deserialize, Serialize};

/// Event data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvalRunSucceededData {
    /// The unique ID of the eval run.
    pub id: String,
}
