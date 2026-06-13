// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookResponseCompletedData` DTO.

use serde::{Deserialize, Serialize};

/// Event data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponseCompletedData {
    /// The unique ID of the model response.
    pub id: String,
}
