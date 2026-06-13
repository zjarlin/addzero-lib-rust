// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookEvalRunFailedData` DTO.

use serde::{Deserialize, Serialize};

/// Event data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvalRunFailedData {
    /// The unique ID of the eval run.
    pub id: String,
}
