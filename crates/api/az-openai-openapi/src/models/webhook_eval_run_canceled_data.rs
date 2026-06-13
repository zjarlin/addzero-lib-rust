// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebhookEvalRunCanceledData` DTO.

use serde::{Deserialize, Serialize};

/// Event data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvalRunCanceledData {
    /// The unique ID of the eval run.
    pub id: String,
}
